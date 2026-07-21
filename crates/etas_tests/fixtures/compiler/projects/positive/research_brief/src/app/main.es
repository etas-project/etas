module app.main;

import std.collections.Array;
import app.research.brief.{make_brief};

flow main(args: Array<string>) -> i32 ![]
{
    let topic = "language runtime";
    let title = make_brief(topic).title;
    return 0;
}
