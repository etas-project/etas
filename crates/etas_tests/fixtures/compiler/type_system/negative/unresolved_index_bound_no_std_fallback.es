module tests.compiler.type_system.negative.unresolved_index_bound_no_std_fallback;

spec Index;

flow id<T ~ Index>(value: T) -> T
{
    return value;
}

flow main(args: Array<string>) -> i32
{
    return id<i32>(1);
}
