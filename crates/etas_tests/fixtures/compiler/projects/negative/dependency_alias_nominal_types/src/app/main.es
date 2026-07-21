module app.main;

import app.consumer.{run};

flow main(args: Array<string>) -> i32
{
    return run();
}
