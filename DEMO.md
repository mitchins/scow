# Scow Application Demo

## Sample UI Layout

```
┌──────────────── File Tree ─────────────────┐┌─────────────── Staging Area ───────────────┐
│                                             ││                                             │
│ 📁 test_directory (55 MiB)                  ││ DELETE: /tmp/test/old_file.txt             │
│   📁 videos (50 MiB)                        ││ MOVE: /tmp/test/photo.jpg → /mnt/backup    │
│     📄 movie.mp4 (50 MiB)                   ││                                             │
│   📁 images (5 MiB)                         ││ Space to free: 1.2 MiB                     │
│     📄 photo.jpg (5 MiB) ➜ /mnt/backup      ││                                             │
│   📁 documents (26 B)                       ││                                             │
│     📄 report.txt (11 B)                    ││                                             │
│   📁 temp (15 B)                            ││                                             │
│     📄 cache.tmp (15 B) [DELETE]            ││                                             │
│                                             ││                                             │
└─────────────────────────────────────────────┘└─────────────────────────────────────────────┘
┌───────────────────────────────────── Controls ──────────────────────────────────────────────┐
│ q: Quit | s: Scan | Space: Mark Delete | m: Move | Enter: Commit                           │
└──────────────────────────────────────────────────────────────────────────────────────────────┘
```

## Features Demonstrated

### Split-Pane Layout
- **Left Pane**: File tree with recursive directory structure
  - Files sorted by size (largest first)
  - Icons for directories (📁) and files (📄)
  - Human-readable size display (MiB, KiB, etc.)
  
- **Right Pane**: Staging area showing pending operations
  - Lists all staged operations
  - Shows operation type (DELETE, MOVE)
  - Displays net size change

### Visual Feedback
- **Yellow text + ➜**: File marked for move operation
- **Red text + [DELETE]**: File marked for deletion
- **Normal text**: No operation staged

### Keyboard Controls
| Key     | Action                                    |
|---------|-------------------------------------------|
| q       | Quit application                          |
| s       | Scan/rescan current directory             |
| Space   | Toggle delete for current file            |
| m       | Open destination picker modal             |
| c       | Clear all staged operations               |
| Enter   | Commit operations (shows confirmation)    |
| y       | Confirm execution (in dialog)             |
| n/Esc   | Cancel operation (in dialog)              |

## Example Workflow

1. **Launch Scow**
   ```bash
   scow /path/to/cleanup
   ```

2. **Browse Files**
   - View directory tree sorted by size
   - Identify large files or directories

3. **Stage Operations**
   - Press **Space** to mark files for deletion
   - Press **m** to select files for moving to another location
   - Multiple operations can be staged

4. **Review Staging**
   - Right pane shows all pending operations
   - See total space that will be freed

5. **Execute**
   - Press **Enter** to commit operations
   - Confirm with **y** in the dialog
   - Operations execute with automatic rescan

## Configuration

Create `~/.config/scow/config.toml`:

```toml
[[destinations]]
name = "External Backup"
path = "/mnt/backup"

[[destinations]]
name = "Remote Server"
path = "user@server.com:/data/archive"
```

## Advanced Features

### Remote Transfers
When moving to a remote destination (with `@` or `ssh:` prefix), Scow automatically uses `rsync`:
```bash
rsync -avz --progress /local/file user@server:/remote/path
```

### Size Calculations
- Directories show cumulative size of all contents
- Staging area tracks net size change
- Negative values = space freed
- Positive values = space used

### Safety Features
- Confirmation dialog before executing operations
- Input validation for remote paths
- Command injection prevention
- Error handling with clear messages
