# Composing Polyglot Servers

In this tutorial, you'll experience FTL's killer feature: seamlessly composing tools written in different programming languages into a single MCP server.

## What You'll Build

Starting from your first project, you'll:
- Add a second tool written in Python
- See how Rust and Python tools work together
- Understand how the WebAssembly Component Model enables language interoperability
- Build a practical multi-language MCP server

## Prerequisites

- Complete the [Your First FTL Project](./first-project.md) tutorial
- Python 3.10+ installed
- Your `my-first-project` from the previous tutorial

## Step 1: Add a Python Tool

Let's add a tool that complements our Rust greeting tool. We'll create a Python tool that generates random facts:

```bash
cd my-first-project
ftl add random-fact --language python
```

This creates a new Python component alongside your existing Rust tool:


You should see:

```
.
├── ftl.toml
├── hello-world
│   ├── Cargo.lock
│   ├── Cargo.toml
│   ├── Makefile
│   └── src
│       └── lib.rs
├── random-fact
│   ├── Makefile
│   ├── pyproject.toml
│   ├── README.md
│   ├── src
│   │   ├── __init__.py
│   │   └── main.py
│   └── tests
│       ├── __init__.py
│       └── test_main.py
└── README.md
```

## Step 2: Implement the Python Tool

Open `random-fact/src/main.py` and replace the generated code:

```python
import random
from ftl_sdk import FTL


ftl = FTL()

FACTS = {
    "animals": [
        "Octopuses have three hearts and blue blood.",
        "A group of flamingos is called a 'flamboyance'.",
        "Wombat poop is cube-shaped.",
        "Sharks are older than trees - they've existed for over 400 million years.",
        "A shrimp's heart is in its head.",
    ],
    "science": [
        "The human brain uses about 20% of the body's total energy.",
        "A single cloud can weigh more than a million pounds.",
        "A day on Venus is longer than its year.",
        "Your stomach gets an entirely new lining every 3-5 days.",
        "The human eye can distinguish about 10 million colors.",
    ],
    "food": [
        "Bananas are berries, but strawberries aren't.",
        "Honey never spoils - archaeologists have found edible honey in ancient Egyptian tombs.",
        "Carrots were originally purple, not orange.",
        "Chocolate was once used as currency by the Aztecs.",
        "Pineapples take about two years to grow.",
    ],
    "history": [
        "The shortest war in history lasted only 38-45 minutes between Britain and Zanzibar in 1896.",
        "The Great Wall of China isn't visible from space with the naked eye.",
        "Cleopatra lived closer in time to the Moon landing than to the construction of the Great Pyramid.",
        "The first computer bug was an actual bug - a moth trapped in a Harvard computer in 1947.",
        "Napoleon was actually average height for his time period.",
    ],
    "space": [
        "There are more possible games of chess than atoms in the observable universe.",
        "A day on Mars is about 24 hours and 37 minutes.",
        "Jupiter has at least 79 known moons.",
        "The Sun makes up 99.86% of our solar system's mass.",
        "One million Earths could fit inside the Sun.",
    ]
}


@ftl.tool
def get_random_fact(category: str = "general") -> str:
    """Get a random interesting fact.
    
    Args:
        category: The category of fact (animals, science, food, history, space, or general for any)
    
    Returns:
        A random fact as a string
    """
    if category == "general":
        # Get a random fact from any category
        all_facts = []
        for facts_list in FACTS.values():
            all_facts.extend(facts_list)
        fact = random.choice(all_facts)
    elif category in FACTS:
        fact = random.choice(FACTS[category])
    else:
        available_categories = list(FACTS.keys()) + ["general"]
        return f"Unknown category '{category}'. Available categories: {', '.join(available_categories)}"
    
    return f"{category.title()} Fact: {fact}"

@ftl.tool
def get_fact_count() -> str:
    """Get the total number of available facts.
    
    Returns:
        The number of facts in the database
    """
    total_count = sum(len(facts_list) for facts_list in FACTS.values())
    category_counts = {category: len(facts) for category, facts in FACTS.items()}
    
    result = f"I know {total_count} interesting facts across {len(FACTS)} categories!\n\n"
    result += "Facts by category:\n"
    for category, count in category_counts.items():
        result += f"  • {category.title()}: {count} facts\n"
    
    return result


IncomingHandler = ftl.create_handler()
```

## Step 3: Build Both Languages

Now let's build our polyglot project:

```bash
ftl build

→ Building 2 components in parallel

  [hello-world] ✓ Built in 0.1s
  [random-fact] ✓ Built in 5.1s                                                                                 
✓ All components built successfully!
```

## Step 4: Test the Polyglot Server

Start your server:

```bash
ftl up
```

Now you have an MCP server with tools written in two different languages!

Restart your MCP client and try them out!

### Claude Code Example

```bash
⏺ I'll test the poly tools now that they've been reconnected.
  ⎿  I know 25 interesting facts across 5 categories!

⏺ polytest - get_random_fact (MCP)(category: "animals")
  ⎿  Animals Fact: Octopuses have three hearts and blue blood.

⏺ polytest - get_random_fact (MCP)(category: "space")
  ⎿  Space Fact: The Sun makes up 99.86% of our solar system's mass.
```

## What's Happening Under the Hood?

This seamless language interoperability is powered by:

1. **WebAssembly Component Model**: Each tool compiles to a standardized WASM component interface
2. **Universal Protocol**: All tools speak the same MCP protocol regardless of implementation language
3. **Spin Framework**: Orchestrates and routes requests between components
4. **FTL Runtime**: Handles lifecycle, security, and communication

## Key Insights

- **Language Choice is Tactical**: Pick the best language for each too
- **No Performance Penalties**: WASM compilation means no interpretation overhead
- **Sandboxed Security**: Each tool runs in its own isolated environment
- **Simple Deployment**: One server, multiple languages, single deployment unit

## What You've Learned

Congratulations! You've just:

- **Built a polyglot MCP server** with tools in multiple languages
- **Experienced true language interoperability** via WebAssembly  
- **Understood the Component Model's role** in language integration  
- **Deployed multiple languages** as a single, cohesive server  

## Next Steps

Now that you've mastered polyglot composition:

- **Understand the Architecture**: Read [Core Concepts](../core-concepts/README.md) to learn how the Component Model works
- **Master the APIs**: Explore [SDK Reference](../../sdk/README.md) for each language
- **Get Inspired**: Browse [Examples](../../examples/README.md) for advanced patterns

You're now ready to build sophisticated MCP servers that leverage the strengths of multiple programming languages!