module tests.compiler.type_system.negative.nominal_record_not_structural;

type Review = {
    accepted: bool,
    notes: string
};

flow publish(review: Review) -> i32
{
    return 0;
}

flow main(args: Array<string>) -> i32
{
    let raw = {
        accepted = true,
        notes = "approved"
    };
    return publish(raw);
}
