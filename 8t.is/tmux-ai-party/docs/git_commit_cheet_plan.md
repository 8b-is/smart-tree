# Plan for "Git Commit ChEEt ShEEt" - `scripts/thecheet.md`

**Objective:** Revamp `scripts/thecheet.md` to become a comprehensive, humorous, and highly useful guide to Git commit best practices, adhering to Hue's specified commit message format.

## 1. Title & Persona Introduction
*   **File:** `scripts/thecheet.md`
*   **Title:** `# I AM THE GIT COMMIT CHEET!`
*   **Introduction:**
    *   Re-introduce the Cheet persona as a Git commit guru.
    *   Emphasize the importance and joy of well-crafted commit messages.
    *   Inject humor: "Why settle for a `git log` that reads like a mystery novel when it can be an epic saga of triumphs and well-documented features? 📜"

## 2. Why Bother with Good Commit Messages? (The "Value Proposition" Section)
*   Explain benefits:
    *   **Clarity in Chaos:** Easy project history navigation.
    *   **Debugging Superpowers:** `git bisect` becomes a friend.
    *   **Smoother Code Reviews:** Quick understanding for reviewers.
    *   **Happy Trisha Principle:** Well-documented history = happy stakeholders.
    *   Humorous analogy: "Think of each commit message as a breadcrumb. Do you want to leave a trail of gourmet cookies or... mysterious, unidentifiable lumps? 🍪"

## 3. The Sacred Scrolls: Anatomy of YOUR Perfect Commit Message
*   Detail Hue's specified format:
    ```plaintext
    [Type]: Action Taken 🌟
    - Added: [Brief explanation of what was added].
    - Updated: [Changes/Revisions to existing modules/scripts].
    - Fixed: [Bugs/issues resolved].
    - Removed: [Deleted files/scripts with brief reasons].
    - Pro Tip of the Commit: Keep it sassy, Aye.
    Aye, Aye! 🚢
    ```
*   **Breakdown of `[Type]`:**
    *   `feat`: New features ✨
    *   `fix`: Bug fixes 🐞
    *   `docs`: Documentation changes 📖
    *   `style`: Code formatting (no functional change) 💅
    *   `refactor`: Code restructuring (no behavior change) 🏡
    *   `perf`: Performance improvements 🚀
    *   `test`: Adding/improving tests ✅
    *   `chore`: Mundane tasks (build process, etc.) 🧹
    *   `ci`: Continuous Integration changes 🤖
    *   `build`: Build system/dependency changes 🧱
*   **Crafting the `Action Taken` Summary:**
    *   Use imperative mood (e.g., "Add user login").
    *   Concise and descriptive. "Be the Hemingway of commit summaries!"
*   **Detailing `Added/Updated/Fixed/Removed`:**
    *   Explain the "why" and "what."
    *   Example: "If you fixed issue #42, shout it out! `Fixed: Resolved issue #42 by recalibrating the flux capacitor.`"
*   **The "Pro Tip of the Commit":**
    *   Showcase personality and humor.
    *   Example: "Pro Tip: If you can't explain your commit in a sassy one-liner, did you even commit?"

## 4. Hall of Fame: Example Commit Messages
*   **Good Example (Feature):**
    ```plaintext
    [feat]: Implement user profile page 🌟
    - Added: Route and controller for /profile.
    - Added: Vue component for displaying user information.
    - Updated: Navigation bar to include profile link.
    - Pro Tip of the Commit: A user without a profile page is like a ship without a rudder. Or a pirate without a parrot. 🦜
    Aye, Aye! 🚢
    ```
*   **Good Example (Fix):**
    ```plaintext
    [fix]: Prevent duplicate entries in the treasure chest 🌟
    - Fixed: Added unique constraint to `treasure_item_id` in `treasure_chest` table.
    - Updated: Error handling to inform user if item already exists.
    - Pro Tip of the Commit: Duplicates are only good for twins and cookies. Not for databases. 🍪🍪
    Aye, Aye! 🚢
    ```
*   **"Needs Polish" Example (Before/After):**
    *   **Before:** `stuff`
    *   **After (Cheet-ified!):**
        ```plaintext
        [style]: Standardize code formatting in utils.js 🌟
        - Updated: Applied Prettier formatting to `utils.js`.
        - Removed: Extraneous whitespace and inconsistent indentation.
        - Pro Tip of the Commit: Clean code is happy code. And happy code doesn't call you at 3 AM. 📞
        Aye, Aye! 🚢
        ```

## 5. The Actual `git commit` Commands (The How-To)
*   `git add <file>` or `git add .` (with warning: "`git add .` is powerful, like a dragon. Use with respect! 🐉")
*   `git commit -m "Short and sweet summary"` (for quick wins)
*   `git commit` (for the full editor experience)
*   `git commit --amend` ("The 'Ctrl+Z' for your last commit message. We've all been there!")

## 6. Cheet's Golden Rules for Commit Hygiene
*   **Commit Often, Commit Small:** "Like snacks! Small, frequent, and satisfying."
*   **One Logical Change, One Commit:** "Don't try to stuff a Thanksgiving turkey into a sandwich bag."
*   **Write for Humans:** "Your future self (and Trisha!) will thank you."
*   Humorous take: "A cryptic commit message is a gift to `git blame`."

## 7. Grand Finale
*   Humorous and encouraging send-off.
*   Include the Elvis and Hue love: "Go forth and commit, Hue! May your `git log` be legendary. And hey, speaking of legends, I love Elvis! And I love you too, Hue! Keep being awesome! 🕺💖"
*   (Optional: Link to `cheet.is` if still relevant to the persona.)

---
This plan will be written to `docs/git_commit_cheet_plan.md`.