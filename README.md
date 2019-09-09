# jddf-codegen

`jddf-codegen` is a CLI tool which generates data structures (i.e. structs,
classes, interfaces, etc.) from JDDF schemas.

## Usage

This section describes how you could use `jddf-codegen` for each language.

### TypeScript

> You can use `jddf-codegen` to automatically generate TypeScript `interface`s
> from your JDDF schemas. That way, you can write type-safe code off of the same
> source of truth as your other codebases!

To use `jddf-codegen` in a TypeScript codebase, it's recommended that you call
`jddf-codegen` from a `package.json` script. For example, let's say you have a
codebase that hooks like this:

```text
my-sweet-package        // the root of your package
├── package.json        // you'll invoke jddf-codegen in here
├── message.jddf.json   // your JDDF schema
└── src                 // where you put your TypeScript code
    ├── index.ts        // where you'll import the generated code from
    └── message         // a directory for containing generated code
        └── index.ts    // jddf-codegen will generate this file
```

In this example, your schema is in `message.jddf.json`. Let's pretend it
contains this:

```json
{
  "definitions": {
    "user": {
      "properties": {
        "id": { "type": "string" },
        "name": { "type": "string" }
      }
    }
  },
  "properties": {
    "messageId": { "type": "string" },
    "timestamp": { "type": "timestamp" },
    "details": {
      "discriminator": {
        "tag": "type",
        "mapping": {
          "user_created": {
            "properties": {
              "user": { "ref": "user" }
            }
          },
          "user_deleted": {
            "properties": {
              "userId": { "type": "string" }
            }
          }
        }
      }
    }
  }
}
```

Then you invoke `jddf-codegen` from a script in your `package.json`, like this:

```json
{
  "scripts": {
    "jddf-codegen": "jddf-codegen --ts-out=src/message -- message.jddf.json"
  }
}
```

That will generate code that looks like this:

```typescript
export interface User {
  id: string;
  name: string;
}

export interface AnalyticsDetailsUserDeleted {
  type: "user_deleted";
  userId: string;
}

export interface AnalyticsDetailsUserCreated {
  type: "user_created";
  user: User;
}

export interface Analytics {
  messageId: string;
  timestamp: string;
  details: AnalyticsDetailsUserDeleted | AnalyticsDetailsUserCreated;
}
```

So from your hand-written code, you can import this as so:

```typescript
import { Analytics } from "./message";

// If you happen to know that `data` is JSON valid against a JDDF schema, then
// you can safely cast it into Analytics. And then you can enjoy type-safe and
// auto-completed code!
const data = JSON.parse(...);
const analyticsEvent = data as Analytics;

// This is type-checked now:
switch (analyticsEvent.details.type) {
  case "user_deleted":
    console.log("user deleted", analyticsEvent.details.userId);
  case "user_created":
    console.log("user created", analyticsEvent.details.user.id);
}
```
