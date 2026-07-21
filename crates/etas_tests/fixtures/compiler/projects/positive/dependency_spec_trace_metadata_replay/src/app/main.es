module app.main;

import app.consumer.{consume_local};
import app.model.{local_packet};

flow main(args: Array<string>) -> i32
{
    let encoded: string = consume_local(local_packet());
    return 0;
}
