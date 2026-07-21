// support: MemoryRegion, Store, MemorySelection, MemoryVersion, MemoryConflict, Memory.read, Memory.write
// layer: interpreter
// polarity: positive
// status: covered-positive
module tests.compiler.std_conformance.interpreter.memory_version_conflict;

import std.memory.{MemoryConflict, version};

alias CaseId = string;
type Case = { status: string };
alias CaseMemorySchema = MemoryRegion<{
    Cases: Store<CaseId, Case>,
}>;

let CaseMemory =
    std.memory.region<CaseMemorySchema>(
        stable_id = "case_memory",
        store = "case-main"
    );

flow update_case(id: CaseId, next: Case) -> string ![Memory.write<CaseMemory>, Error<MemoryConflict>] {
    CaseMemory.Cases.put(id, Case { status = "existing" });
    let stale = version("0");
    CaseMemory.Cases.put_versioned(id, next, stale);
    return "written";
}

flow main(args: Array<string>) -> i32 ![Memory.write<CaseMemory>] {
    let result = handle {
        update_case("case-1", Case { status = "next" })
    } with {
        Error<MemoryConflict>.raise(conflict) => {
            finish "conflict";
        }
    };
    if result == "conflict" {
        return 0;
    }
    return 1;
}
