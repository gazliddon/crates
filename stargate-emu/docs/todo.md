# Getting Started

> Handoff: see `docs/pickup.md` for the full state of recent work (gazm
> plugin, nvim config, gazm releases roadmap) and next actions.

## Setup

- [x] Assess the quality of docs in this doc directory
    - It's my first time doing this, I'd like this to be as idiomatic as
      possible
- [x] Is the emulator spec doc accurate? Adjust where it isn't
    - [x] Create a doc that specifies the locations of where everything is

## Analysis

- [ ] Understand this crate and others it uses
- [ ] Create docs to describe what it's doing
- [ ] There are some testing tools already, are they good? What else would I
      need?
- [ ] Give me instructions for launching and testing

## Get Working with Gazm

- [ ] Fix symbol and map loading - the format that gazm spits out has changed
    - [ ] Learn how to assemble stargate - it's in the `../stargate` directory
- [ ] Any other work needed to get it to launch

## Plan

- [ ] Let's do some planning


# Goals

To write a stargate emulator using my 6809 and 6800 emu packages and any new
ones that need to be writer.

The emulator has a debugger that allows for source level debugging and includes
the features

- break
- Single step
- Step over
- breakpoints and breakpoint management

I want to concentrate just on running stargate for now but longer term I want to
make am emulator harness and start adding more machines
