module app.release.plan;

import std.collections.Array;

public type Release = {
    version: string,
    changes: Array<string>,
};

public flow build_release(version: string) -> Release ![] {
    return Release {
        version = version,
        changes = ["api", "ui"],
    };
}
