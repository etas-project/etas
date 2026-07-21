module app.research.model;

import std.collections.Array;

public type Citation = {
    title: string,
    url: string,
};

public type Brief = {
    title: string,
    citations: Array<Citation>,
};
