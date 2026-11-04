module library;
import helper.{First, Second};

public spec Reader { flow read(self) -> i32 ![]; }
public type Item = {};
impl Item ~ Reader {
    flow read(self) -> i32 ![] { return 21; }
}
