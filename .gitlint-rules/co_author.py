"""The rule that a Claude co-author trailer names the model."""

import re

from gitlint.rules import CommitRule, RuleViolation

BARE = re.compile(r"co-authored-by:\s*claude\s*<", re.I)


class CoAuthorNamesModel(CommitRule):
    """A `Co-Authored-By: Claude` trailer names the model, as AGENTS.md tells."""

    name = "co-author-names-model"
    id = "UC1"

    def validate(self, commit):
        return [
            RuleViolation(self.id, "Name the model, as in 'Claude Opus 5.5'", line, n)
            for n, line in enumerate(commit.message.body, 2)
            if BARE.match(line)
        ]
