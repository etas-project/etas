module app.todo.renderer;

import app.todo.model.{TaskList};

public flow render_summary(tasks: TaskList) -> string ![] {
    return "todo summary";
}
