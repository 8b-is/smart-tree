---
title: Git Commit ChEEt ShEEt!
description: Your hilarious and indispensable guide to crafting legendary Git commit messages that even Trisha from Accounting will admire!
published: true
date: 2025-06-17T00:00:00.000Z
tags: git, commit, best practices, version control, humor, cheet sheet
editor: markdown
dateCreated: 2025-06-17T00:00:00.000Z
---

# I AM THE GIT COMMIT CHEET! 🚀

Ahoy there, Matey! It's your friendly neighborhood Git Commit Cheet, here to navigate you through the treacherous waters of version control. My mission? To make your `git log` so legendary, so crystal clear, that future-you (and a very impressed Trisha from Accounting) will build statues in your honor! 🗿

Why settle for a `git log` that reads like a mystery novel when it can be an epic saga of triumphs and well-documented features? 📜 Let's make some magic!

## Why Bother with Good Commit Messages? 🤔 (The "Sell Me This Pen...cil Sharpener" Section)

"But Cheet," I hear you cry, "isn't `git commit -m 'stuff'` good enough?" Oh, my sweet summer child, let me illuminate the path to enlightenment:

*   **Clarity in Chaos:** Imagine searching for that *one* change you made three months ago. Good commit messages turn a frantic treasure hunt into a leisurely stroll in the park. 🌳
*   **Debugging Superpowers:** When `git bisect` is your only hope, clear messages are the beacons guiding you to that pesky bug. Less hair-pulling, more high-fiving! 🙌
*   **Smoother Sailing for Code Reviews:** Help your esteemed colleagues (and your pal, Hue!) understand your genius with minimal head-scratching. They'll thank you, probably with cookies. 🍪
*   **The Happy Trisha Principle:** A well-documented project history makes Trisha's day. And a happy Trisha means... well, let's just say the spreadsheets align favorably. 📈
*   **Pro Tip from the Cheet:** Think of each commit message as a breadcrumb. Do you want to leave a trail of gourmet cookies or... mysterious, unidentifiable lumps? Choose wisely, young Padawan.

## The Sacred Scrolls: Anatomy of YOUR Perfect Commit Message 📜✍️

Behold, the sacred format, handed down through generations of wise coders (and approved by Hue!):

```plaintext
[Type]: Action Taken 🌟
- Added: [Brief explanation of what was added].
- Updated: [Changes/Revisions to existing modules/scripts].
- Fixed: [Bugs/issues resolved].
- Removed: [Deleted files/scripts with brief reasons].
- Pro Tip of the Commit: Keep it sassy, Aye.
Aye, Aye! 🚢
```

Let's dissect this magnificent beast:

### `[Type]`: The Flavor of Your Commit!

This little prefix tells everyone the *kind* of awesome you just did:

*   `feat`: A fantastic new feature! ✨ (e.g., `[feat]: Add user login via unicorn magic`)
*   `fix`: Squashing those pesky bugs! 🐞 (e.g., `[fix]: Prevent squirrels from eating the database backups`)
*   `docs`: Polishing the sacred texts (documentation). 📖 (e.g., `[docs]: Clarify that our API runs on hopes and dreams`)
*   `style`: Making code pretty (no functional change, just pure artistry). 💅 (e.g., `[style]: Align all the curly braces, for serenity`)
*   `refactor`: Restructuring code without changing its behavior (like a home makeover for your functions!). 🏡 (e.g., `[refactor]: Untangle the spaghetti code in payment_processor.js`)
*   `perf`: Speeding things up! Making it go VROOOM! 🚀 (e.g., `[perf]: Optimize image loading by compressing pixels with sheer willpower`)
*   `test`: Adding or improving tests (because testing is caring, and caring is sharing... bugs before they hit production). ✅ (e.g., `[test]: Ensure the 'launch_rocket' function doesn't actually launch a real rocket`)
*   `chore`: Mundane but necessary tasks (build process, dependency updates, etc.). 🧹 (e.g., `[chore]: Update left-pad to version 1,000,001`)
*   `ci`: Continuous Integration magic. Making robots do our bidding! 🤖 (e.g., `[ci]: Configure pipeline to deploy on Tuesdays if it's sunny`)
*   `build`: Changes to the build system or external dependencies. 🧱 (e.g., `[build]: Add webpack plugin to convert coffee into code`)

### `Action Taken 🌟`: Your Commit's Headline!

*   **Use the imperative mood:** "Add feature" not "Added feature" or "Adds feature." Think of it as giving Git a command. "Git, you magnificent machine, `Add user authentication`!"
*   **Keep it concise yet descriptive:** Be the Hemingway of commit summaries. Short, punchy, and tells the story.
*   **Capitalize the first letter.** Just like a proper headline.
*   **No period at the end.** It's a title, not a sentence.

### `Added/Updated/Fixed/Removed`: The Juicy Details!

This is where you elaborate on your heroic deeds:

*   Use bullet points for multiple changes.
*   Explain the "what" and a little bit of the "why" if it's not obvious.
*   Example: `Fixed: Resolved issue #42 by recalibrating the flux capacitor and offering it a soothing cup of tea.`

### `Pro Tip of the Commit`: Unleash Your Inner Sage (or Comedian)!

This is your moment to shine, to drop some wisdom, a joke, or a sassy remark.
*   Example: "Pro Tip: If you can't explain your commit in a sassy one-liner, did you even commit? 🤔"

### `Aye, Aye! 🚢`: The Signature of a True Code Navigator!

Seals the deal. Every. Single. Time.

## Hall of Fame: Example Commit Messages 🏆

Let's see some masterpieces in action!

### Glorious Example (Feature):

```plaintext
[feat]: Implement user profile page with customizable avatars 🌟
- Added: Route and controller for /user/profile.
- Added: Vue component for displaying and editing user information.
- Added: Avatar upload functionality with preview.
- Updated: Navigation bar to include a dynamic link to the user's profile.
- Pro Tip of the Commit: A user without a profile page is like a ship without a rudder. Or a pirate without a parrot. Or a web developer without coffee. 🦜☕
Aye, Aye! 🚢
```

### Stellar Example (Fix):

```plaintext
[fix]: Prevent duplicate entries in the treasure_chest table 🌟
- Fixed: Added unique constraint to `treasure_item_id` in the `treasure_chest` database table.
- Updated: Error handling in `addTreasureItem` service to inform the user if the item already exists (politely, of course).
- Pro Tip of the Commit: Duplicates are only good for twins and cookies. Not for your precious database entries. 🍪🍪
Aye, Aye! 🚢
```

### "Could Use Some Polish" Example (and how to buff it up!):

**Before (The Dark Ages):**
`git commit -m "stuff"`

**After (The Renaissance, Cheet-ified!):**
```plaintext
[style]: Standardize code formatting across all JavaScript files 🌟
- Updated: Applied Prettier formatting rules to all `.js` files in the `src/` directory.
- Removed: Extraneous whitespace, inconsistent indentation, and rogue semicolons.
- Pro Tip of the Commit: Clean code is happy code. And happy code doesn't call you at 3 AM with existential crises. 📞😴
Aye, Aye! 🚢
```

## The Actual `git commit` Commands (The Nitty-Gritty How-To) 🛠️

Alright, captain, here's how you steer the ship:

1.  **Stage your changes:**
    *   `git add <file_name>` (to add a specific file)
    *   `git add .` (to add all changes in the current directory and subdirectories. Powerful, like a dragon. Use with respect! 🐉)

2.  **Commit your changes:**
    *   **For short messages (the quick dinghy ride):**
        `git commit -m "[feat]: Add quick save button 🌟"`
        (Remember to follow THE FORMAT, even for short ones!)
    *   **For longer, more detailed messages (the majestic galleon voyage):**
        `git commit`
        This will open your default text editor, where you can craft your multi-line masterpiece according to **The Sacred Scrolls**.

3.  **Oops, Made a Typo in My Last Commit Message? (The "Ctrl+Z" for Commits):**
    *   `git commit --amend`
        This lets you rewrite your *very last* commit message. If you haven't pushed yet, it's like it never happened! (Shhh, it's our little secret. 😉)
        You can also use it to add files you forgot: `git add forgotten_file.js && git commit --amend`

## Cheet's Golden Rules for Commit Hygiene ✨🧼

Follow these, and your `git log` will be the envy of developers everywhere:

*   **Commit Often, Commit Small:** Think of commits like snacks! Small, frequent, and satisfying. Don't wait until you've written a novel to save your work.
*   **One Logical Change, One Commit:** Don't try to stuff a Thanksgiving turkey, a Christmas tree, and a birthday cake into one commit. Each commit should represent a single, cohesive change.
*   **Write for Humans (Especially Future You):** Remember, the person who will benefit most from clear commit messages is often yourself, six months down the line, wondering what on earth past-you was thinking. Be kind to future-you.
*   **A Cryptic Commit Message is a Gift to `git blame`... and Not in a Good Way:** Don't make your teammates (or Trisha!) play detective. 🕵️

## Grand Finale! 🎉

  visit https://developer.nvidia.com/cuda-downloads for a different distro/os
     ```sh
     wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2404/x86_64/cuda-keyring_1.1-1_all.deb
     sudo dpkg -i cuda-keyring_1.1-1_all.deb
     sudo apt-get update
     sudo apt-get -y install cuda-toolkit-12-5
     ```

  2. **Install NVIDIA Docker Toolkit**:

     ```sh
      curl -fsSL https://nvidia.github.io/libnvidia-container/gpgkey | sudo gpg --dearmor -o /usr/share/keyrings/nvidia-container-toolkit-keyring.gpg \
      && curl -s -L https://nvidia.github.io/libnvidia-container/stable/deb/nvidia-container-toolkit.list | \
      sed 's#deb https://#deb [signed-by=/usr/share/keyrings/nvidia-container-toolkit-keyring.gpg] https://#g' | \
      sudo tee /etc/apt/sources.list.d/nvidia-container-toolkit.list
      sudo apt-get update
      sudo apt-get install -y nvidia-container-toolkit
     ```

  ### On the Container

  1. **Set Variables for CUDA Version and Distribution**:
     ```sh
     CUDA_VERSION=12.5.1
     DISTRIBUTION=ubuntu24.04
     ```

  2. **Pull the NVIDIA CUDA Container**:
     ```sh
     docker pull nvidia/cuda:${CUDA_VERSION}-runtime-${DISTRIBUTION}
     ```

  3. **Run the NVIDIA CUDA Container**:
     ```sh
     docker run --gpus all -it nvidia/cuda:${CUDA_VERSION}-runtime-${DISTRIBUTION} /bin/bash
     ```

  4. **Verify CUDA Installation**:
     ```sh
     nvidia-smi
     ```

     - **Optional**: Install `nvtop` for a better GPU monitoring experience:
       ```sh
       sudo apt-get update
       sudo apt-get install -y cmake libncurses5-dev git
       git clone https://github.com/Syllo/nvtop.git
       mkdir -p nvtop/build && cd nvtop/build
       cmake ..
       make
       sudo make install
       nvtop
       ```

  ## Troubleshooting Tips

  1. **Container Not Starting**:
     - Ensure the Docker daemon is running: `sudo systemctl start docker`
     - Check logs for errors: `docker logs container_id`

  2. **Image Pull Failing**:
     - Verify network connectivity.
     - Ensure you have the correct image name and tag.

  3. **Permission Issues**:
     - Use `sudo` if necessary.
     - Add your user to the Docker group: `sudo usermod -aG docker $USER`

  4. **GPU Not Detected**:
     - Verify NVIDIA drivers are installed on the host.
     - Ensure Docker is configured to use NVIDIA runtime.

  >   NVIDIA GPU work sometimes involves a reboot.  It seems to magically fix things.
  {.is-info}


  ## Conclusion

  With these quick solutions and setups, you'll have Docker running smoothly in no time. Embrace the power of simplicity and efficiency with Docker!

  ---

  Did you enjoy this cheat sheet? Visit [cheet.is](https://cheet.is) for more fun and practical guides. Happy Dockering!

---

I include tags to make my pages easily searchable.

I like to add some humor as most people are frustrated once they get to me.

I have many awards in simplifying everything.  Chris & Alex love what I do.  They helped me get this awesome job!
