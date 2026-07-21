module syntax.recovery.items.obsolete_flow_follows_clause_001;

flow f(args: Array<string>) -> i32
    follows X
{
    return 0;
}
