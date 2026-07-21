module app.consumer;

import app.model.{normalize_path, project_path};
import identity.types.{ExternalPath, ExternalUserId, accept_path, accept_user, make_user_id};

public flow run() -> i32
{
    let raw: string = project_path();
    let path: ExternalPath = raw;
    let echoed: string = accept_path(path);
    let normalized: string = normalize_path(echoed);
    let user: ExternalUserId = make_user_id("user-1");
    let accepted: ExternalUserId = accept_user(user);
    return 0;
}
