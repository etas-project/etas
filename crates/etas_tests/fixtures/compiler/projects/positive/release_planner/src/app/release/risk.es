module app.release.risk;

import app.release.plan.{Release};

public type ReleaseRisk = {
    level: string,
    score: i32,
};

public flow score_release(release: Release) -> ReleaseRisk ![] {
    return ReleaseRisk { level = "medium", score = 5 };
}
