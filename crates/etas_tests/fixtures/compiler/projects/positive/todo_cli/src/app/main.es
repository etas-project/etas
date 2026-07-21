module app.main;

import std.collections.Array;
import app.todo.model.{Task, TaskList};
import app.todo.parser.{parse_tasks};
import app.todo.renderer.{render_summary};

flow main(args: Array<string>) -> i32 ![]
{
    let input = "buy milk; ship package";
    let tasks = parse_tasks(input);
    let summary = render_summary(tasks);
    return 0;
}
