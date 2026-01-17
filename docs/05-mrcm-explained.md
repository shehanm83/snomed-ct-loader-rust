# MRCM (Machine Readable Concept Model)

## What is MRCM?

MRCM defines validation rules for SNOMED CT expressions. It specifies:

1. **What attributes** can be used with which concepts
2. **What values** those attributes can have
3. **How many times** an attribute can appear (cardinality)

## Why MRCM Matters

When creating postcoordinated expressions (combining concepts), MRCM ensures validity:

```
Valid:   404684003 |Clinical finding| : 363698007 |Finding site| = 39057004 |Lung|
Invalid: 404684003 |Clinical finding| : 363698007 |Finding site| = 73211009 |Diabetes|
                                                                    ↑ Not a body structure!
```

## MRCM Components

### 1. Domain (MrcmDomain)

Defines semantic domains where attributes apply.

```
Domain: Clinical finding (404684003)
├── Includes: << 404684003 |Clinical finding|
├── Allows attributes: Finding site, Severity, etc.
└── Template: [[+id(< 404684003)]] : [[0..*]] { ... }
```

**Key Fields:**
- `domain_constraint` - ECL defining domain membership
- `domain_template_for_precoordination` - Template for authoring
- `domain_template_for_postcoordination` - Template for runtime use

### 2. Attribute Domain (MrcmAttributeDomain)

Defines which attributes are valid in which domains.

```
Attribute: Finding site (363698007)
├── Valid in domain: Clinical finding
├── Grouped: Yes (must be in role group)
├── Cardinality: 0..* (zero or more overall)
├── In-group cardinality: 0..1 (at most one per group)
└── Rule strength: Mandatory
```

**Key Fields:**
- `domain_id` - Which domain this rule applies to
- `grouped` - Must attribute be in a role group?
- `attribute_cardinality` - Overall count constraint
- `attribute_in_group_cardinality` - Per-group count constraint
- `rule_strength_id` - Mandatory vs Optional

### 3. Attribute Range (MrcmAttributeRange)

Defines valid values for attributes.

```
Attribute: Finding site (363698007)
├── Range: << 123037004 |Body structure|
└── Meaning: Only body structures are valid values
```

**Key Fields:**
- `range_constraint` - ECL defining valid values
- `attribute_rule` - Additional validation logic

## Cardinality

Cardinality constrains occurrence counts:

| Pattern | Meaning | Example Use |
|---------|---------|-------------|
| `0..*` | Zero or more | Finding site (can have multiple) |
| `0..1` | Optional single | Laterality (at most one) |
| `1..1` | Required single | Must have exactly one |
| `1..*` | One or more | At least one required |

### In Code

```rust
use snomed_types::mrcm::Cardinality;

let card = Cardinality::parse("0..1")?;
assert!(card.allows(0));  // true - optional
assert!(card.allows(1));  // true - within range
assert!(!card.allows(2)); // false - exceeds max

let unbounded = Cardinality::unbounded(); // 0..*
assert!(unbounded.allows(100)); // true
```

## Role Groups

Role groups bundle related attributes:

```
Concept: Bacterial pneumonia
├── Group 0 (ungrouped):
│   └── IS_A → Infectious disease of lung
│
├── Group 1:
│   ├── Finding site → Right lung structure
│   └── Causative agent → Streptococcus
│
└── Group 2:
│   ├── Finding site → Left lung structure
│   └── Causative agent → Staphylococcus
```

**Why groups?**
- Attributes in same group are related
- Allows multiple occurrences of the same pattern
- Cardinality applies per-group and overall

## MRCM Files in RF2

| File Pattern | Content |
|--------------|---------|
| `der2_cRefset_MRCMDomainSnapshot_*.txt` | Domain definitions |
| `der2_cRefset_MRCMAttributeDomainSnapshot_*.txt` | Attribute-domain mappings |
| `der2_cRefset_MRCMAttributeRangeSnapshot_*.txt` | Attribute value ranges |

## Example: Validating an Expression

Expression to validate:
```
404684003 |Clinical finding| : 363698007 |Finding site| = 39057004 |Lung|
```

Validation steps:

1. **Check domain**: Is `404684003` a valid domain?
   - Look up in MrcmDomain
   - ✓ Yes, "Clinical finding" is a domain

2. **Check attribute allowed**: Is `Finding site` valid for this domain?
   - Look up in MrcmAttributeDomain where domain = Clinical finding
   - ✓ Yes, Finding site (363698007) is allowed

3. **Check value range**: Is `Lung` a valid value for Finding site?
   - Look up in MrcmAttributeRange for Finding site
   - Range constraint: `<< 123037004 |Body structure|`
   - Is 39057004 a descendant of Body structure?
   - ✓ Yes, Lung is a body structure

4. **Check cardinality**: Is the count valid?
   - Finding site cardinality: 0..*
   - We have 1 occurrence
   - ✓ 1 is within 0..*

Result: ✓ Expression is valid

## ECL (Expression Constraint Language)

MRCM uses ECL for constraints:

| ECL | Meaning |
|-----|---------|
| `<< 404684003` | Descendants of Clinical finding (including self) |
| `< 404684003` | Descendants of Clinical finding (excluding self) |
| `>> 404684003` | Ancestors of Clinical finding (including self) |
| `> 404684003` | Ancestors of Clinical finding (excluding self) |
| `*` | Any concept |

Example range constraints:
```
<< 123037004 |Body structure|     → Any body structure
< 410607006 |Organism|            → Any organism (not root)
<< 272379006 |Event| OR << 71388002 |Procedure| → Events or Procedures
```
