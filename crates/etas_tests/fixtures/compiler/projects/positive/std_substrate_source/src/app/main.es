module app.main;

import std.collections.Array;
import app.substrate.service.{summarize_substrate};

flow main(args: Array<string>) -> i32
{
    let report = summarize_substrate();
    if report.count > 0 {
        return 0;
    }
    return 1;
}
