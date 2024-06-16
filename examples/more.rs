use derive_more::{Add, Display, From};

fn main() -> anyhow::Result<()> {
    let my_int = MyInt::from(1);
    let myenum = MyEnum::from(my_int);
    println!("{:?}", myenum);
    Ok(())
}

#[derive(Debug, PartialEq, Eq, From, Add, Display, Clone, Copy)]
struct MyInt(i32);

#[derive(Debug, From, Add, Display)]
enum MyEnum {
    Int(MyInt),
    Uint(u32),
    #[display(fmt = "Nothing")]
    Nothing,
}
