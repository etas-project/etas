// support: MemoryRegion, Store, MemorySelection, Memory.read, Memory.write
// layer: effect
// polarity: positive
// status: covered-positive
module tests.compiler.std_conformance.effect.memory_store_effects;

type PaperId = string;
type Paper = { title: string, body: string };
alias ProjectMemorySchema = MemoryRegion<{
    Papers: Store<PaperId, Paper>,
}>;

let ProjectMemory =
    std.memory.region<ProjectMemorySchema>(
        stable_id = "project_memory",
        store = "project-main"
    );

flow find_paper(id: PaperId) -> Option<Paper> ![Memory.read<ProjectMemory>] {
    return ProjectMemory.Papers.get(id);
}

flow save_paper(id: PaperId, paper: Paper) -> unit ![Memory.write<ProjectMemory>] {
    ProjectMemory.Papers.put(id, paper);
}

flow upsert_paper(id: PaperId, paper: Paper) -> unit ![Memory.read<ProjectMemory>, Memory.write<ProjectMemory>] {
    ProjectMemory.Papers.upsert(id, paper);
}

flow recent_titles(limit: usize) -> MemorySelection<Paper> ![Memory.read<ProjectMemory>] {
    return ProjectMemory.Papers.scan().limit(Tokens(limit));
}

flow main(args: Array<string>) -> i32 {
    return 0;
}
