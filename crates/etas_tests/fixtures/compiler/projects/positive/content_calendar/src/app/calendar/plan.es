module app.calendar.plan;

import std.collections.Array;

public type CalendarItem = {
    channel: string,
    title: string,
};

public type CalendarPlan = {
    items: Array<CalendarItem>,
};

public flow default_plan() -> CalendarPlan ![] {
    return CalendarPlan {
        items = [
            CalendarItem { channel = "blog", title = "launch notes" },
        ],
    };
}
