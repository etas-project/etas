module app.main;

import std.collections.Array;
import app.calendar.plan.{default_plan};
import app.calendar.publish.{publish_plan};

flow main(args: Array<string>) -> i32 ![]
{
    let published = publish_plan(default_plan());
    return 0;
}
