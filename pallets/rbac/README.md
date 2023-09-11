# Pallet RBAC (Role Based Access Management)

## Overview

The RBAC Pallet provides functionalities to create, manage, and assign roles with specific permissions to different accounts. Using this pallet, developers can define a set of permissions for different roles and assign these roles to various accounts, effectively creating a role-based access control system.

Coupled with a signed extension it could be used for runtime calls validation right before their dispatch.

### Features

- **Role Creation and Removal**: Create and remove roles with specific names.
- **Role Assignment and Unassignment**: Assign and unassign roles to and from accounts.
- **Permission Management**: Add and remove specific calls (permissions) to and from roles.

### Examples

Here is how you can use the features of the RBAC pallet (examples contain encoded calls, which could be easily dispatched in polkadot.js/apps using Developer->Extrinsics->Decode<->Submission tabs):

#### Creating a role
Create a new role "Remarker" (requires sudo):
```
0x060007002052656d61726b65720000
```

#### Adding a call to a role
Add System::remark_with_event call to the role "Remarker" (requires sudo):
```
0x060007012052656d61726b657200070431
```

#### Assigning a role to a user
Assign role "Remarker" to user Alice (requires sudo):
```
0x06000703d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d2052656d61726b6572
```

#### Unassign role
Unassign Alice from role "Remarker" (requires sudo):
```
0x06000704d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d2052656d61726b6572
```

#### Remove call from a role
Remove System::remark_with_event call from role "Remarker" (requires sudo):
```
0x060007022052656d61726b657200070431
```

#### Remove a role entirely
Remove role "Remarker" (requires sudo):
```
0x060007052052656d61726b6572
```

#### Dispatch remark_with_event with Remarker role
Dispatch a call:
```
0x0706000704312052656d61726b6572
```
