#![no_std]
extern crate alloc;

use wie_jvm_support::WieJavaClassProto;

pub mod classes;

/// LG Telecom's MMPP API: the OEM classes ez-java (다운타운) MIDlets use on top of MIDP 1.0.
pub fn get_protos() -> [WieJavaClassProto; 8] {
    [
        classes::mmpp::lang::MathFP::as_proto(),
        classes::mmpp::media::BackLight::as_proto(),
        classes::mmpp::media::MediaPlayer::as_proto(),
        classes::mmpp::media::Vibration::as_proto(),
        classes::mmpp::microedition::lcdui::GraphicsX::as_proto(),
        classes::mmpp::microedition::lcdui::TextFieldX::as_proto(),
        classes::mmpp::phone::ContentsManager::as_proto(),
        classes::mmpp::phone::Phone::as_proto(),
    ]
}
