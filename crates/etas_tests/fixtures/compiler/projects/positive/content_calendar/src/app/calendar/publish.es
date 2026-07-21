module app.calendar.publish;

import app.calendar.plan.{CalendarPlan};

public flow publish_plan(plan: CalendarPlan) -> string ![] {
    return "scheduled";
}
