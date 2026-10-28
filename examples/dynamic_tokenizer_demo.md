# 🧬 Dynamic Tokenizer Demo

## The Problem

Every project has its own vocabulary:
- React projects: `components/`, `hooks/`, `contexts/`, `.jsx`
- Django projects: `views/`, `models/`, `migrations/`, `.py`
- Rust projects: `src/`, `tests/`, `benches/`, `.rs`

Hardcoding tokens for every possible project structure is impossible!

## The Solution: Dynamic Learning

Smart Tree's dynamic tokenizer analyzes your specific project and learns its patterns.

### Example 1: React Project

```bash
st --mode quantum-semantic my-react-app/
```

The tokenizer discovers:
```
TOKENS:
  80=components/
  81=.jsx
  82=hooks/
  83=useAuth
  84=useState
  85=Button
  86=contexts/
  87=providers/
  88=__tests__/
```

Output:
```
${80}Button.jsx:F4,F7x3
${80}UserList.jsx:F4,F0x5,{84}x3
${82}{83}.js:F0,{84},F7
```

### Example 2: Django Project

```bash
st --mode quantum-semantic my-django-site/
```

The tokenizer discovers:
```
TOKENS:
  80=apps/
  81=views.py
  82=models.py
  83=migrations/
  84=tests.py
  85=admin.py
  86=serializers.py
  87=urls.py
```

Output:
```
#{80}users/{82}:F4x3,F5x10
#{80}users/{81}:F5x8,F4
#{80}users/{84}:F8,F5x15
```

### Example 3: Mixed Language Project

```bash
st --mode quantum-semantic full-stack-app/
```

The tokenizer adapts to multi-language patterns:
```
TOKENS:
  80=frontend/
  81=backend/
  82=components/
  83=.tsx
  84=api/
  85=routes/
  86=models/
  87=.py
  88=__pycache__/
```

## Compression Results

| Project Type | Files | Original Size | Compressed | Reduction |
|-------------|-------|---------------|------------|-----------|
| React App | 500 | 25KB | 2.5KB | 90% |
| Django Site | 300 | 18KB | 1.6KB | 91% |
| Rust Crate | 100 | 8KB | 0.7KB | 91.3% |
| Mixed Stack | 1000 | 45KB | 3.8KB | 91.6% |

## How It Works

1. **Pattern Discovery**: Analyzes all file paths and names
2. **Frequency Analysis**: Counts occurrences of each pattern
3. **Token Assignment**: Assigns shorter tokens to more frequent patterns
4. **Smart Compression**: Replaces patterns with tokens in output

## Benefits

- **Zero Configuration**: No need to define project-specific rules
- **Adaptive**: Learns from YOUR specific project structure
- **Efficient**: Most frequent patterns get shortest tokens
- **Universal**: Works with any programming language or framework

As Omni says: "Every project speaks its own language. Now Smart Tree can learn them all!" 🚀