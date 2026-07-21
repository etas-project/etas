module app.main;

import std.collections.Array;
import app.support.case.{CaseRequest};
import app.support.router.{route_case};

flow main(args: Array<string>) -> i32 ![]
{
    let body = "demo support request";
    let request = CaseRequest { customer_id = "demo", body = body };
    let queue = route_case(request);
    return 0;
}
