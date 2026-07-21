module app.main;

import std.collections.Array;
import app.knowledge.index.{sample_index};
import app.knowledge.search.{search};

flow main(args: Array<string>) -> i32 ![]
{
    let query = "release";
    let title = search(sample_index(), query).title;
    return 0;
}
