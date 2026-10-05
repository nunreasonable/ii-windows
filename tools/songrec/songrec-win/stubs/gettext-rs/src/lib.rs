//! Identity gettext(): messages stay in English.

pub fn gettext<T: Into<String>>(msgid: T) -> String {
    msgid.into()
}
