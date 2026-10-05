pub fn gettext<T: Into<String>>(msgid: T) -> String {
    msgid.into()
}
