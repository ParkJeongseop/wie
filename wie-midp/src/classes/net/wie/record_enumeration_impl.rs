use alloc::{vec, vec::Vec};

use bytemuck::cast_vec;

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::rms::{RecordComparator, RecordFilter, RecordStore};

// class net.wie.RecordEnumerationImpl
//
// The enumeration RecordStore.enumerateRecords hands out: the ids of the records that pass the
// filter, in comparator order (ascending id without one). `index` is the position of the record
// returned last, -1 before the first call and after a reset.
pub struct RecordEnumerationImpl;

impl RecordEnumerationImpl {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "net/wie/RecordEnumerationImpl",
            parent_class: Some("java/lang/Object"),
            interfaces: vec!["javax/microedition/rms/RecordEnumeration"],
            methods: vec![
                JavaMethodProto::new(
                    "<init>",
                    "(Ljavax/microedition/rms/RecordStore;Ljavax/microedition/rms/RecordFilter;Ljavax/microedition/rms/RecordComparator;Z)V",
                    Self::init,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("numRecords", "()I", Self::num_records, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("nextRecord", "()[B", Self::next_record, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("nextRecordId", "()I", Self::next_record_id, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("previousRecord", "()[B", Self::previous_record, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("previousRecordId", "()I", Self::previous_record_id, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("hasNextElement", "()Z", Self::has_next_element, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("hasPreviousElement", "()Z", Self::has_previous_element, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("reset", "()V", Self::reset, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("rebuild", "()V", Self::rebuild, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("keepUpdated", "(Z)V", Self::keep_updated, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("isKeptUpdated", "()Z", Self::is_kept_updated, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("destroy", "()V", Self::destroy, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("store", "Ljavax/microedition/rms/RecordStore;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("filter", "Ljavax/microedition/rms/RecordFilter;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("comparator", "Ljavax/microedition/rms/RecordComparator;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("ids", "[I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("index", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("keptUpdated", "Z", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        store: ClassInstanceRef<RecordStore>,
        filter: ClassInstanceRef<RecordFilter>,
        comparator: ClassInstanceRef<RecordComparator>,
        keep_updated: bool,
    ) -> JvmResult<()> {
        tracing::debug!("net.wie.RecordEnumerationImpl::<init>({this:?}, {store:?}, {filter:?}, {comparator:?}, {keep_updated})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        jvm.put_field(&mut this, "store", "Ljavax/microedition/rms/RecordStore;", store).await?;
        jvm.put_field(&mut this, "filter", "Ljavax/microedition/rms/RecordFilter;", filter)
            .await?;
        jvm.put_field(&mut this, "comparator", "Ljavax/microedition/rms/RecordComparator;", comparator)
            .await?;
        jvm.put_field(&mut this, "keptUpdated", "Z", keep_updated).await?;

        jvm.invoke_virtual(&this, "net/wie/RecordEnumerationImpl", "rebuild", "()V", ()).await
    }

    async fn rebuild(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.RecordEnumerationImpl::rebuild({this:?})");

        let store: ClassInstanceRef<RecordStore> = jvm.get_field(&this, "store", "Ljavax/microedition/rms/RecordStore;").await?;
        let filter: ClassInstanceRef<RecordFilter> = jvm.get_field(&this, "filter", "Ljavax/microedition/rms/RecordFilter;").await?;
        let comparator: ClassInstanceRef<RecordComparator> = jvm.get_field(&this, "comparator", "Ljavax/microedition/rms/RecordComparator;").await?;

        let database = RecordStore::get_database(jvm, context, &store).await?;
        let mut record_ids = database.get_record_ids().await;
        record_ids.sort_unstable();

        // (id, contents); the contents are only read for a filter or a comparator
        let mut records: Vec<(i32, ClassInstanceRef<Array<i8>>)> = Vec::with_capacity(record_ids.len());
        for record_id in record_ids {
            if filter.is_null() && comparator.is_null() {
                records.push((record_id as i32, None.into()));
                continue;
            }
            let Some(data) = database.get(record_id).await else {
                continue;
            };
            let mut contents = jvm.instantiate_array("B", data.len()).await?;
            jvm.store_array(&mut contents, 0, cast_vec::<u8, i8>(data)).await?;
            let contents: ClassInstanceRef<Array<i8>> = contents.into();

            if !filter.is_null() {
                let matches: bool = jvm
                    .invoke_virtual(&filter, "javax/microedition/rms/RecordFilter", "matches", "([B)Z", (contents.clone(),))
                    .await?;
                if !matches {
                    continue;
                }
            }
            records.push((record_id as i32, contents));
        }

        if !comparator.is_null() {
            // The comparator is application code, so every comparison is a call into the JVM; an
            // insertion sort keeps the order stable with few of them for the handful of records.
            let mut sorted: Vec<(i32, ClassInstanceRef<Array<i8>>)> = Vec::with_capacity(records.len());
            for record in records {
                let mut position = sorted.len();
                while position > 0 {
                    let order: i32 = jvm
                        .invoke_virtual(
                            &comparator,
                            "javax/microedition/rms/RecordComparator",
                            "compare",
                            "([B[B)I",
                            (record.1.clone(), sorted[position - 1].1.clone()),
                        )
                        .await?;
                    if order != Self::PRECEDES {
                        break;
                    }
                    position -= 1;
                }
                sorted.insert(position, record);
            }
            records = sorted;
        }

        let ids = records.iter().map(|record| record.0).collect::<Vec<_>>();
        let mut array = jvm.instantiate_array("I", ids.len()).await?;
        jvm.store_array(&mut array, 0, ids).await?;
        jvm.put_field(&mut this, "ids", "[I", array).await?;
        jvm.put_field(&mut this, "index", "I", -1).await?;

        Ok(())
    }

    const PRECEDES: i32 = -1;

    async fn ids(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<Vec<i32>> {
        let ids: ClassInstanceRef<Array<i32>> = jvm.get_field(this, "ids", "[I").await?;
        let length = jvm.array_length(&ids).await?;

        jvm.load_array(&ids, 0, length).await
    }

    async fn num_records(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("net.wie.RecordEnumerationImpl::numRecords({this:?})");

        Ok(Self::ids(jvm, &this).await?.len() as i32)
    }

    async fn has_next_element(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("net.wie.RecordEnumerationImpl::hasNextElement({this:?})");

        let count = Self::ids(jvm, &this).await?.len() as i32;
        let index: i32 = jvm.get_field(&this, "index", "I").await?;

        Ok(index != count - 1)
    }

    async fn has_previous_element(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("net.wie.RecordEnumerationImpl::hasPreviousElement({this:?})");

        let count = Self::ids(jvm, &this).await?.len() as i32;
        let index: i32 = jvm.get_field(&this, "index", "I").await?;

        Ok(count != 0 && index != 0)
    }

    async fn next_record_id(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("net.wie.RecordEnumerationImpl::nextRecordId({this:?})");

        let ids = Self::ids(jvm, &this).await?;
        let index: i32 = jvm.get_field(&this, "index", "I").await?;
        let Some(record_id) = ids.get((index + 1) as usize) else {
            return Err(jvm.exception("javax/microedition/rms/InvalidRecordIDException", "No next record").await);
        };
        jvm.put_field(&mut this, "index", "I", index + 1).await?;

        Ok(*record_id)
    }

    async fn previous_record_id(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("net.wie.RecordEnumerationImpl::previousRecordId({this:?})");

        let ids = Self::ids(jvm, &this).await?;
        let index: i32 = jvm.get_field(&this, "index", "I").await?;
        // before the first call the enumeration is also "after the last record"
        let previous = if index == -1 { ids.len() as i32 - 1 } else { index - 1 };
        if previous < 0 {
            return Err(jvm
                .exception("javax/microedition/rms/InvalidRecordIDException", "No previous record")
                .await);
        }
        jvm.put_field(&mut this, "index", "I", previous).await?;

        Ok(ids[previous as usize])
    }

    async fn next_record(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Array<i8>>> {
        tracing::debug!("net.wie.RecordEnumerationImpl::nextRecord({this:?})");

        let record_id: i32 = jvm
            .invoke_virtual(&this, "net/wie/RecordEnumerationImpl", "nextRecordId", "()I", ())
            .await?;

        Self::record(jvm, &this, record_id).await
    }

    async fn previous_record(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Array<i8>>> {
        tracing::debug!("net.wie.RecordEnumerationImpl::previousRecord({this:?})");

        let record_id: i32 = jvm
            .invoke_virtual(&this, "net/wie/RecordEnumerationImpl", "previousRecordId", "()I", ())
            .await?;

        Self::record(jvm, &this, record_id).await
    }

    async fn record(jvm: &Jvm, this: &ClassInstanceRef<Self>, record_id: i32) -> JvmResult<ClassInstanceRef<Array<i8>>> {
        let store: ClassInstanceRef<RecordStore> = jvm.get_field(this, "store", "Ljavax/microedition/rms/RecordStore;").await?;

        jvm.invoke_virtual(&store, "javax/microedition/rms/RecordStore", "getRecord", "(I)[B", (record_id,))
            .await
    }

    async fn reset(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.RecordEnumerationImpl::reset({this:?})");

        jvm.put_field(&mut this, "index", "I", -1).await
    }

    async fn keep_updated(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, keep_updated: bool) -> JvmResult<()> {
        tracing::debug!("net.wie.RecordEnumerationImpl::keepUpdated({this:?}, {keep_updated})");

        jvm.put_field(&mut this, "keptUpdated", "Z", keep_updated).await?;
        if keep_updated {
            // there are no change notifications from the store; rebuilding now is the closest match
            let _: () = jvm.invoke_virtual(&this, "net/wie/RecordEnumerationImpl", "rebuild", "()V", ()).await?;
        }

        Ok(())
    }

    async fn is_kept_updated(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("net.wie.RecordEnumerationImpl::isKeptUpdated({this:?})");

        jvm.get_field(&this, "keptUpdated", "Z").await
    }

    async fn destroy(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.RecordEnumerationImpl::destroy({this:?})");

        Ok(())
    }
}
