module tests.compiler.type_system.positive.nominal_record_accessor;

type Review = {
    title: string,
    score: i32,
};

flow review() -> Review
{
    return Review { title = "spec", score = 5 };
}

flow title(value: Review) -> string
{
    return value.title;
}

flow score(value: Review) -> i32
{
    return value.score;
}

flow main(args: Array<string>) -> i32
{
    let item = review();
    let name: string = title(item);
    let points: i32 = score(item);
    return 0;
}
