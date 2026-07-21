module app.effects.service;

import std.io.{println};
import app.effects.model.{EffectReport, build_report};

public flow run_effect_project(seed: i32) -> i32 ![Error<IOError>]
{
    let report = build_report(seed);
    println(report.name);
    return report.count;
}
