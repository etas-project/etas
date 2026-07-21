// support: MemoryRegion, Store, MemorySelection, Memory.read, Memory.write, Map
// layer: effect
// polarity: negative
// status: blocked-by-impl
// expect: Map<K, V> cannot replace Store<K, V>
// expect: memory resource handle must be an immutable std.memory.region resource
// expect: upsert must require Memory.read and Memory.write
module tests.compiler.std_conformance.effect.memory_bad_resource_shapes;

type PaperId = string;
type Paper = { title: string, body: string };
alias BadMemorySchema = MemoryRegion<{
    Papers: Map<PaperId, Paper>,
}>;

let BadMemory =
    std.memory.region<BadMemorySchema>(
        stable_id = "bad_memory",
        store = "bad-main"
    );

flow bad_upsert(id: PaperId, paper: Paper) -> unit ![Memory.write<BadMemory.Papers>] {
    BadMemory.Papers.upsert(id, paper);
}

flow main(args: Array<string>) -> i32 {
    return 0;
}
