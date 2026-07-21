module app.todo.parser;

import app.todo.model.{Task, TaskList};

public flow parse_tasks(input: string) -> TaskList ![] {
    return TaskList {
        items = [
            Task { title = input, done = false },
        ],
    };
}
