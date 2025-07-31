use std::fmt::Write;

use sea_query::Iden;

macro_rules! table {
    ($table_name:expr, $enum_name:ident { $($variant_name:ident,)* }) => {
        #[derive(Iden)]
        pub enum $enum_name {
            $($variant_name),*
        }

        impl $enum_name {
            pub fn table_name() -> impl Iden {
                struct TableName {}

                impl Iden for TableName {
                    fn unquoted(&self, s: &mut dyn Write) {
                        write!(s, "{}", $table_name).unwrap()
                    }
                }

                TableName {}
            }

            pub fn column_name(self) -> String {
                self.to_string()
            }
        }
    };
}

table!(
    "users",
    Users {
        Uuid,
        Username,
        Password,
        Email,
        JwtId,
        IsAdmin,
    }
);
