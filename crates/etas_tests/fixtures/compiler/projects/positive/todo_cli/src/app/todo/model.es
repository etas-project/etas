module app.todo.model;

import std.collections.Array;

public type Task = {
    title: string,
    done: bool,
};

public type TaskList = {
    items: Array<Task>,
};
