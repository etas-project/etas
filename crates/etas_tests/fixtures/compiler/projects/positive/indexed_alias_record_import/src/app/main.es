module app.main;

import app.checks.{score};
import app.model.{sample_generic_queue, sample_match_result, sample_queue};

flow main(args: Array<string>) -> i32
{
    return score(sample_match_result(), sample_queue(), sample_generic_queue(), 0);
}
