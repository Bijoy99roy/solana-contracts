# Staking Program

## Learnings:

- Sol can only be transfered by accounts owned by system program.
- Anchor doesn't let any pda to be assigned as SystemProgram inside accounts
- One of the solution to efficiently store and send solana in a program pda is by using a different pda that only stores sol (There might be some better options, have to explore)
- Storing bumps as part of account only costs 1 byte and saves a lot of on chain computation 