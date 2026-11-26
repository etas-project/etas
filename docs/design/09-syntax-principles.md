# Source Grammar

The EBNF describes source forms. Type, effect, scope and API contracts are
specified in the linked chapters. Metavariables are not additional keywords.
Source examples use four-space indentation.

## 1. Lexical Rules

- `//` comments end at the newline; `/* ... */` comments end at the first
  `*/` and do not nest. Comments have no semantic effect.
- Strings and characters support `\\u{...}` Unicode scalar escapes with one
  to six hexadecimal digits.
- Names such as `Result`, `Index`, `infer` and `join` are resolved standard
  contracts, not keywords. See [Library Boundaries](01-overview.md#4-language-and-library-boundaries).
- The `^` reflection extension is specified [separately](18-reflection-and-checked-code-generation.md).

## 2. Grammar

### 2.1 Modules And Items

```ebnf
Program       ::= ModuleDecl? ImportDecl* Item*

ModuleDecl    ::= "module" Path ";"
ImportDecl    ::= Visibility? "import" ImportTree ";"
ImportTree    ::= Path ImportTail?
ImportTail    ::= "as" Ident
                | ".*"
                | ".{" ImportTreeList "}"
ImportTreeList
              ::= ImportTreeEntry ("," ImportTreeEntry)* ","?
ImportTreeEntry
              ::= Ident ImportTail?

Item          ::= Annotation* Visibility? ExportableItemDecl
                | Annotation* ImplDecl

Annotation    ::= "@" Path AnnotationArgs?
AnnotationArgs
              ::= "(" AnnotationArgList? ")"
AnnotationArgList
              ::= AnnotationArg ("," AnnotationArg)* ","?
AnnotationArg ::= Expr
                | Ident "=" Expr

ExportableItemDecl
              ::= AliasDecl
                | TypeDecl
                | EnumDecl
                | SpecDecl
                | TopLevelLetDecl
                | EffectDecl
                | ToolDecl
                | AgentDecl
                | ProtocolDecl
                | FlowDecl
```

Imports, visibility, annotation placement and constant-argument checks follow [General Constructs](02-general-programming-constructs.md#10-modules-packages-and-imports).

### 2.2 Types

```ebnf
AliasDecl     ::= "alias" Ident TypeParams? "=" TypeExpr ";"

TypeDecl      ::= "type" Ident TypeParams? ";"  // exactly an empty nominal record: type T = {};
                | "type" Ident TypeParams? "=" TypeExpr ";"
                | "type" Ident TypeParams? "=" RecordType

EnumDecl      ::= "enum" Ident TypeParams? "{" EnumVariants? "}"
EnumVariants  ::= EnumVariant ("," EnumVariant)* ","?
EnumVariant   ::= Ident
                | Ident "(" VariantParams? ")"
                | Ident "{" VariantFields? "}"
VariantParams ::= VariantParam ("," VariantParam)* ","?
VariantParam  ::= (Ident ":")? TypeExpr
VariantFields ::= VariantField ("," VariantField)* ","?
VariantField  ::= Ident ":" TypeExpr

SpecDecl     ::= "spec" Ident TypeParams? SpecKindAnn? SpecDeclTail
SpecKindAnn  ::= ":" SpecKind
SpecKind     ::= "type" | "callable" | "trace"
SpecDeclTail ::= SpecEntailment? (";" | SpecBlock)
                | TypeExpr "=>" TypeExpr EffectSuffix? ";"
                | "=" SpecExpr ";"
SpecEntailment
              ::= "~" BoundList
SpecBlock    ::= "{" SpecItem* "}"
SpecItem     ::= Visibility? SpecFlowSig
SpecFlowSig  ::= "flow" Ident TypeParams? ParamList ReturnType ";"

SpecExpr     ::= SpecOr
SpecOr       ::= SpecAnd ("|" SpecAnd)*
SpecAnd      ::= SpecUnary ("&" SpecUnary)*
SpecUnary    ::= ("+" | "-") ActionPattern
                | SpecTemporal
SpecTemporal ::= SpecPrimary ((">>" | "<<") SpecPrimary)?
SpecPrimary  ::= SpecRef
                | ActionPattern
                | "(" SpecExpr ")"
ActionPattern::= Path StaticEffectArgs?

ImplDecl      ::= TypeImplDecl | SpecImplDecl
TypeImplDecl  ::= "impl" TypeParams? ImplTarget "{" ImplItem* "}"
SpecImplDecl ::= "impl" TypeParams? (TypeExpr | ImplTarget) DeclarationConformance (";" | "{" SpecImplItem* "}")
ImplTarget    ::= Path ImplArgs?
ImplArgs      ::= "<" ImplArg ("," ImplArg)* ","? ">"
ImplArg       ::= TypeExpr | Ident "~" BoundList
ImplItem      ::= Annotation* Visibility? FlowDecl
                | Annotation* Visibility? ToolDecl
                | EffectActionDecl
SpecImplItem ::= Annotation* Visibility? (FlowDecl | ToolDecl)

TypeExpr      ::= HandlerType
                | ArrowType

ArrowType     ::= PrimaryType "->" ArrowType EffectSuffix?
                | PrimaryType

HandlerType   ::= "!" "[" EffectList HandlerTail? "]"
HandlerTail   ::= HandlerProduced? HandlerResult?
HandlerProduced
              ::= "=>" HandlerOutput
HandlerOutput ::= EffectList | "[" "]"
HandlerResult ::= "for" TypeExpr

PrimaryType   ::= PrimitiveType
                | ExistentialType
                | Path TypeArgs?
                | RecordType
                | TupleType
                | "(" TypeExpr ")"

ExistentialType
              ::= "?" "~" BoundList

PrimitiveType ::= "bool" | "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
                | "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
                | "f32" | "f64" | "char" | "string" | "bytes" | "unit" | "never"

RecordType    ::= "{" FieldDecl* "}"
FieldDecl     ::= Visibility? Ident ":" TypeExpr ("=" Expr)? ","?
                | InjectAnnotation Visibility? Ident ":" TypeExpr ","?
InjectAnnotation
              ::= "@" "inject" ("(" EffectRef ")")?
Visibility    ::= "private" | "public"

TupleType     ::= "(" TypeList ")"
TypeList      ::= TypeExpr ("," TypeExpr)* ","?
TypeParams    ::= "<" TypeParam ("," TypeParam)* ","? ">"
TypeParam     ::= Ident TypeParamBound?
                | "effect" Ident TypeParamBound?
TypeParamBound::= "~" BoundList
BoundList     ::= Bound ("+" Bound)*
Bound         ::= SpecRef
TypeArgs      ::= "<" TypeList ">"

SpecSatisfaction
              ::= "~" SpecRef
SpecRef      ::= Path TypeArgs?
DeclarationConformance
              ::= "~" DeclarationConformanceRef
DeclarationConformanceRef
              ::= Path TypeArgs?
                | "(" SpecExpr ")"

EffectSuffix  ::= "!" EffectRow
EffectRow     ::= "[" EffectList? "]"
EffectList    ::= EffectRef ("," EffectRef)* ","?
EffectRef     ::= Path StaticEffectArgs?
StaticEffectArgs
              ::= "<" StaticEffectArgList? ">"
StaticEffectArgList
              ::= StaticEffectArg ("," StaticEffectArg)* ","?
StaticEffectArg
              ::= TypeExpr
                | Literal
                | StableResourcePath
                | ConstPath
                | "_"
```

```ebnf
EffectDecl    ::= "effect" Ident TypeParams? ("extends" Effect)? (";" | EffectBlock)
EffectBlock   ::= "{" EffectActionDecl* "}"
EffectActionDecl
              ::= "action" Ident TypeParams? ParamList ReturnType ";"
```

Kinds follow [Types](05-type-system-and-errors.md). Field defaults/injection are confined to nominal record/agent declarations, not structural type expressions. Omitted fields use [initialization rules](02-general-programming-constructs.md#7-composite-types); `@inject` uses [dependency requests](02-general-programming-constructs.md#72-scoped-injection).

### 2.3 Top-Level Bindings

```ebnf
TopLevelLetDecl
              ::= "let" Ident TypeAnnot? "=" Expr ";"
TypeAnnot     ::= ":" TypeExpr
```

Module bindings are immutable; their initialization cannot execute ordinary runtime actions. Declarative resource handles follow [Top-Level Bindings](02-general-programming-constructs.md#31-top-level-bindings).

### 2.4 Flows

```ebnf

FlowDecl      ::= "flow" Ident TypeParams? ParamList ReturnType? FlowClause* FlowBody FlowHandler?
FlowBody      ::= Block
                | "=" Expr ";"?
FlowHandler   ::= "with" HandlerArg
ReturnType    ::= "->" TypeExpr EffectSuffix?

ParamList     ::= "(" ParamListInner? ")"
ParamListInner::= Param ("," Param)* ","?
                | ReceiverParam ("," Param)* ","?
Param         ::= Ident ":" TypeExpr DefaultArg?
                | InjectAnnotation Ident ":" TypeExpr
DefaultArg    ::= "=" Expr
ReceiverParam ::= "self" | "var" "self"

FlowClause    ::= DeclarationConformance
```

Parameter eligibility, defaults/injection and receiver modes follow [General Constructs](02-general-programming-constructs.md#41-default-and-injected-parameters); grammar reuse does not permit injected lambda/action parameters. Behavior summaries are not source clauses.

### 2.5 Tools, Agents, Persistent Resources

```ebnf
ToolDecl      ::= "tool" Path ParamList "->" TypeExpr EffectSuffix? ToolClause* ToolBody
ToolBody      ::= Block | ";"
ToolClause    ::= DeclarationConformance

AgentDecl     ::= "agent" Ident TypeParams? "=" RecordType ";"?
```

Agent blocks contain private fields only; methods are explicit `impl` members. Bodyless tools require a resolved imported implementation. See [Agents And Tools](03-agents-tools-prompts-memory.md).

### 2.6 Trace Specs And Protocols

```ebnf
EffectPattern ::= Path EffectPatternArgs?
EffectPatternArgs
              ::= "<" EffectPatternArgList? ">"
EffectPatternArgList
              ::= EffectPatternArg ("," EffectPatternArg)* ","?
EffectPatternArg
              ::= Ident "~" BoundList
                | StaticEffectArg
                | "_"

ProtocolDecl  ::= "protocol" Ident "{" ProtocolMsg* "}"
ProtocolMsg   ::= Path "->" Path ":" TypeExpr ";"
```

Action selectors are static, not captures of runtime arguments. Trace operators apply only to trace specs; protocols describe role/message exchange. See [Trace Specs](05-type-system-and-errors.md#154-trace-specs).

### 2.7 Statements

```ebnf
Block         ::= "{" Stmt* FinalExpr? "}"
FinalExpr     ::= Expr

Stmt          ::= LetStmt
                | VarStmt
                | AssignStmt
                | IfStmt
                | MatchStmt
                | ForStmt
                | WhileStmt
                | RetryStmt
                | ResumeStmt
                | FinishStmt
                | ReturnStmt
                | BreakStmt
                | ContinueStmt
                | BlockStmt
                | ExprStmt

LetStmt       ::= "let" Pattern TypeAnnot? "=" Expr ";"
VarStmt       ::= "var" Pattern TypeAnnot? "=" Expr ";"
AssignStmt    ::= Expr "=" Expr ";"
TypeAnnot     ::= ":" TypeExpr

HeadExpr      ::= Expr  // record expressions require grouping at head delimiter depth
IfStmt        ::= "if" HeadExpr Block ("else" (IfStmt | Block))?
MatchStmt     ::= "match" HeadExpr "{" MatchArm* "}"
MatchArm      ::= Pattern "=>" (Expr | Block) ","?

LocalAnnotation
              ::= Annotation  // checked: only the recognized @limits contract
ForStmt       ::= LocalAnnotation* "for" Pattern "in" HeadExpr Block
WhileStmt     ::= LocalAnnotation* "while" HeadExpr Block
RetryStmt     ::= LocalAnnotation* "retry" Block
AnnotatedBlock
              ::= LocalAnnotation+ Block
BlockStmt     ::= Block | AnnotatedBlock

ResumeStmt    ::= "resume" Expr? ";"
FinishStmt    ::= "finish" Expr ";"

ReturnStmt    ::= "return" Expr? ";"
BreakStmt     ::= "break" ";"
ContinueStmt  ::= "continue" ";"
ExprStmt      ::= Expr ";"
```

`resume` and `finish` are control statements; `?` does not replace a statement terminator. Only `@limits` attaches to loops/retry/blocks. See [Handlers](06-effect-system-and-inference.md) and [Resource Contracts](04-flows-human-gates-and-protocols.md#14-resource-contracts-and-budgeted-loops).

### 2.8 Expressions

```ebnf
Expr          ::= Literal
                | Path
                | RecordExpr
                | TupleExpr
                | ArrayExpr
                | ListExpr
                | MapExpr
                | SetExpr
                | RangeExpr
                | CallExpr
                | MethodCallExpr
                | SpecMethodCallExpr
                | PerformExpr
                | HandleExpr
                | HandlerExpr
                | StageComposeExpr
                | PipelineExpr
                | FieldExpr
                | IndexExpr
                | SliceExpr
                | TryExpr
                | UnaryExpr
                | BinaryExpr
                | IfExpr
                | MatchExpr
                | LambdaExpr
                | AnnotatedBlock
                | Block

CallExpr      ::= Expr TypeArgs? "(" ArgList? ")"
MethodCallExpr::= Expr "." Ident TypeArgs? "(" ArgList? ")"
SpecMethodCallExpr
              ::= Expr "::" SpecRef "." Ident "(" ArgList? ")"
PerformExpr   ::= "perform" Effect "." Ident StaticEffectArgs? "(" ArgList? ")"
                | "perform" "infer" TypeArgs? "(" ArgList? ")"
HandleExpr    ::= "handle" Expr "with" HandlerArg
                | Expr "with" HandlerArg
HandlerArg    ::= HandlerBlock | HandlerValueExpr
HandlerValueExpr
              ::= Expr  // must type-check as a handler value; a bare block is invalid
HandlerExpr   ::= "handler" HandlerBlock
HandlerBlock  ::= "{" HandlerArm* Ident? "}"  // optional final dependency shorthand
HandlerArm    ::= Effect "." Ident StaticEffectArgs? "(" PatternList? ")" ContinuationBinder? "=>" HandlerArmBody
                | InjectionHead "=>" Expr (";" | ",")
                | Ident (";" | ",")
InjectionHead ::= TypeExpr | Ident ":" TypeExpr  // bare Ident: type name or field label
ContinuationBinder
              ::= "as" Ident
HandlerArmBody
              ::= ResumeStmt | FinishStmt | Block | Expr ";"
StageComposeExpr
              ::= PipelineStage "|" PipelineStage ("|" PipelineStage)*
PipelineExpr  ::= Expr "~>" PipelineStage ("~>" PipelineStage)*
PipelineStage ::= Expr
LambdaExpr    ::= ParamList "=>" (Expr | Block)
                | Ident "=>" (Expr | Block)

ArgList       ::= Arg ("," Arg)* ","?
Arg           ::= Ident "=" Expr | Expr

RecordExpr    ::= Path? "{" FieldInit* "}"  // type head: construct; record-value head: update
FieldInit     ::= Ident ("=" Expr)? ","?
TupleExpr     ::= "(" ArgList? ")"  // one unnamed expression without a comma is grouping
ArrayExpr     ::= "[" CommaExprList? "]"
ListExpr      ::= "[" SemiExprList? "]"
                | Expr "::" Expr
MapExpr       ::= "{" MapEntryList? "}"
MapEntryList  ::= MapEntry ("," MapEntry)* ","?
MapEntry      ::= Expr "=>" Expr
SetExpr       ::= "#{" CommaExprList? "}"
RangeExpr     ::= "[" Expr "," Expr ")"
                | "(" Expr "," Expr "]"

CommaExprList ::= Expr ("," Expr)* ","?
SemiExprList  ::= Expr (";" Expr)* ";"?

FieldExpr     ::= Expr "." Ident
IndexExpr     ::= Expr "[" Expr "]"
SliceExpr     ::= Expr "[" Expr "," Expr ")"
                | Expr "(" Expr "," Expr "]"
TryExpr       ::= Expr "?"
UnaryExpr     ::= UnaryOp Expr
BinaryExpr    ::= Expr BinaryOp Expr
IfExpr        ::= "if" HeadExpr Block "else" Block
MatchExpr     ::= MatchStmt

UnaryOp       ::= "!" | "-"
BinaryOp      ::= "==" | "!=" | "<" | "<=" | ">" | ">="
                | "+" | "-" | "*" | "/" | "%"
                | "&&" | "||"
```

Arrows associate right; stage operators left. A braceless action arm is one expression or explicit `resume`/`finish` statement, independent of line wrapping; other statements require a block. Expression bodies/block tails implicitly resume; normal fallthrough without a tail is invalid. `never` paths do not resume. See [completion rules](06-effect-system-and-inference.md#22-handler-scope) and [stage rules](04-flows-human-gates-and-protocols.md).

At control-head delimiter depth, `{` begins the following block/arms; group
record expressions in parentheses. Elsewhere the resolved head kind selects
construction or update. Value-headed updates are expressions, not patterns; see
[Composite Types](02-general-programming-constructs.md#7-composite-types).

### 2.9 Names And Patterns

```ebnf
Path          ::= Ident ("." Ident)*
PathList      ::= Path (";" Path)* ";"?

Pattern       ::= Ident
                | "_"
                | LiteralPattern
                | TuplePattern
                | ListPattern
                | RecordPattern
                | VariantPattern

LiteralPattern
              ::= "true"
                | "false"
                | IntegerLiteral
                | StringLiteral
                | CharLiteral

TuplePattern  ::= "(" Pattern ("," Pattern)* ","? ")"
ListPattern   ::= "[" "]"
                | PatternAtom "::" Pattern
PatternAtom   ::= Ident | "_" | LiteralPattern | TuplePattern
                | RecordPattern | VariantPattern | "[" "]"
RecordPattern ::= Path? "{" PatternField* "}"
PatternField  ::= Ident (":" Pattern)? ","?
VariantPattern::= Path
                | Path "(" PatternList? ")"
                | Path "{" VariantPatternFields? "}"
VariantPatternFields
              ::= PatternFieldEntry ("," PatternFieldEntry)* ","?
PatternFieldEntry
              ::= Ident (":" Pattern)?
PatternList   ::= Pattern ("," Pattern)* ","?
```

Literal patterns match constants; named-field patterns list each field once. Injection never supplies pattern fields. Match coverage, constructors and scope rules follow [Types](05-type-system-and-errors.md#112-algebraic-data-types-and-recursion).
