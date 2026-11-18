# Spirula Studio

<!-- impeccable:product-schema 1 -->

## Platform

Native desktop. The application uses Dear ImGui and GLFW; it is not a web page
or a mobile application.

## Product Purpose

Spirula Studio prepares image datasets, trains Gaussian splats, and provides
native viewing, editing, rendering, and meshing tools in one C++ application.
CUDA and Vulkan are interchangeable compute backends for training.

## Users and Operating Context

The working audience inferred from the shipped tools is people building and
inspecting 3D reconstructions. They move between captures, prepared datasets,
training runs, models, and saved projects; existing recent-item categories and
file formats remain authoritative. This redesign adds no new account, cloud,
or project-storage model.

## Capabilities and Constraints

- Keep existing import, preparation, training, batch, viewing, editing, render,
  mesh, save, and export behavior reachable.
- Screen navigation must respect active workers, engine ownership, stop
  confirmations, and unsaved edit/render documents.
- Keep the existing embedded fonts, thirteen languages, UI-scale preference,
  persistent layout values, and translated widget IDs.
- The native C++ implementation and both compute backends remain. No additional
  runtime or build dependency is introduced for the interface.
- Use the official development build scripts; patented features remain off.

## Brand Commitments

Keep the Spirula Studio name and factual product copy. The user authorized an
overall UI redesign after the training-layout PR; the earlier layout fixes are
part of that redesign. The user subsequently pinned Blender as the reference
for the whole interface: neutral dark editor surfaces, compact controls,
right-hand properties, and a dominant scene area. Preserve Spirula's product
identity, task names, native workflows, and embedded font coverage.

## Evidence on Hand

The repository contains the real application, localized interface catalogs,
embedded fonts, and existing artwork. Native screenshots under
`docs/notes/images/` show a synthetic three-camera dataset; their generic paths
are demonstration data. GPU-heavy workflows require a supported compute device.

## Product Principles

- Keep the current task, document, and run state legible across every workspace.
- Prefer direct manipulation and familiar native controls for repeated work.
- Distinguish navigating, configuring, running, and inspecting.
- Preserve user input and make destructive transitions explicit.
- Let smaller windows and larger text change structure instead of hiding actions.
