pub trait SqlTable {
    fn name() -> &'static str;
    fn fields() -> Vec<&'static str>;
    fn sql_columns() -> Vec<&'static str>;
}
