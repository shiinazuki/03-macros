#[derive(Debug, macros_derive::TypeName)]
pub enum Value {
    Int(i64),
    Text(String),
    Nothing,
    Pait { left: i64, right: i64 },
}

#[cfg(test)]
mod tests {
    use super::Value;

    #[test]
    fn type_name_works() {
        assert_eq!(Value::type_name(), "Value");
    }
}
