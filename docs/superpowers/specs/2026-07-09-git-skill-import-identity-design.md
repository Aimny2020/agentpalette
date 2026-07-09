# Git Skill 导入身份设计

## 1. 目标

修复 Git Skill 仓库同名时无法并存的问题。典型场景是 `anthropics/skills` 和 `mattpocock/skills` 的仓库名都叫 `skills`，但它们应当可以同时导入、更新、启用到项目，并且不会互相覆盖。

本设计将 Git Skill 的“来源身份”和“安装身份”分离：

- 来源身份用于判断是否是同一个 Git 仓库。
- 安装身份用于决定全局目录、项目目录、数据库主键和成员 Skill ID。

## 2. 非目标

- 不修改仓库内部文件、`SKILL.md`、frontmatter 或子目录结构。
- 不改变本地文件夹导入的命名规则。
- 不实现多个版本或多个分支的同源并存。
- 不从目录名反推 Git URL。
- 不把显示名作为唯一身份。

## 3. 核心规则

Git Skill 导入后必须使用稳定的 `owner-repo` 作为安装身份。

示例：

```text
github.com/anthropics/skills   -> anthropics-skills
github.com/mattpocock/skills   -> mattpocock-skills
github.com/obra/superpowers    -> obra-superpowers
```

最终规则：

```text
Git source identity  = normalized_source
Git install identity = owner-repo
Display name         = metadata.name 优先，缺失时用 owner-repo
Global folder        = ~/.agent-forge/skills/{skill_id}
Project folder       = <project>/.agents/skills/{skill_id}
Update identity      = source_url + tracked_ref + installed_commit
```

## 4. 来源身份

`normalized_source` 继续用于 Git 同源去重。同一个仓库的不同 URL 写法必须归一为同一个值。

示例：

```text
https://github.com/anthropics/skills
git@github.com:anthropics/skills.git
github.com/anthropics/skills
```

都归一为：

```text
github.com/anthropics/skills
```

如果数据库中已经存在相同 `normalized_source`，导入检查返回重复信息，确认导入按钮保持禁用，不创建第二份副本。

## 5. 安装身份

Git 导入的 `skill_id` 必须由 `normalized_source` 的最后两段生成。

```text
normalized_source = github.com/anthropics/skills
owner             = anthropics
repo              = skills
skill_id          = anthropics-skills
```

安装身份生成规则：

- 取 `normalized_source` 的最后两段作为 `owner` 和 `repo`。
- 使用 `owner-repo` 拼接。
- 统一转小写。
- 允许字符为 `a-z`、`0-9`、`-`、`_`、`.`。
- 其他字符转换为 `-`。
- 连续 `-` 可以压缩为单个 `-`。
- 生成结果不能为空。

如果生成的 `skill_id` 对应目录已存在，但 `normalized_source` 不同，不能覆盖。系统应生成确定性短后缀：

```text
anthropics-skills-a1b2c3
```

短后缀来自 `normalized_source` 的稳定 hash。该规则只处理极少数本地目录冲突，不替代默认 `owner-repo` 规则。

## 6. 全局导入行为

Git 导入时：

```text
source_url        = 用户输入的原始 URL
normalized_source = 归一化后的 Git 来源
skill_id          = owner-repo 安装身份
```

全局目录必须使用：

```text
~/.agent-forge/skills/{skill_id}
```

例如：

```text
~/.agent-forge/skills/anthropics-skills/
~/.agent-forge/skills/mattpocock-skills/
```

导入过程：

1. 归一化 Git URL。
2. 按 `normalized_source` 检查是否同源重复。
3. 生成 `skill_id`。
4. clone 到临时 staging 目录。
5. 扫描并校验 `SKILL.md`。
6. 将 staging 目录移动到 `~/.agent-forge/skills/{skill_id}`。
7. 写入 `skill_packages` 记录。

## 7. 导入预览

导入检查结果应返回实际安装身份，前端需要展示它。

后端 `ImportInspection` 建议新增字段：

```text
install_id: string
normalized_source?: string
```

前端导入弹窗显示：

```text
名称：{metadata.name}
安装 ID：anthropics-skills
Git 来源：github.com/anthropics/skills
类型：技能扩展包 / 独立 Skill
成员数：N
```

如果是重复来源，继续显示：

```text
已安装为 {duplicate_skill_id}，不会创建重复副本。
```

重复来源仍然禁用确认导入。

## 8. 显示名规则

界面显示名不能作为唯一身份。

显示名优先级：

```text
manifest / SKILL.md metadata.name
-> owner-repo
-> repo
```

因此本地目录可以是：

```text
mattpocock-skills
```

但卡片显示名仍可来自 Skill 元数据，例如：

```text
mattpocock-skills
```

或仓库自己声明的其他名称。

## 9. 成员 Skill ID

成员 Skill ID 必须跟随包级 `skill_id`。

示例：

```text
anthropics-skills::root
anthropics-skills::skills/foo
mattpocock-skills::root
mattpocock-skills::skills/foo
```

这样成员说明、搜索结果、详情页导航和项目启用关系都不会因两个仓库同名而冲突。

## 10. 项目启用行为

项目启用目录必须跟随全局 `skill_id`。

全局目录：

```text
~/.agent-forge/skills/anthropics-skills/
```

项目目录：

```text
<project>/.agents/skills/anthropics-skills/
```

同一个项目可以同时启用：

```text
<project>/.agents/skills/anthropics-skills/
<project>/.agents/skills/mattpocock-skills/
```

复制到项目时只改变外层目录名，不修改仓库内部内容、`SKILL.md` 或子目录。

项目启用、禁用、同步和移除都必须按 `skill_id` 操作，不能按原始仓库名 `repo` 操作。

## 11. Git 更新行为

Git 更新不能依赖目录名反推远端地址。

更新必须使用数据库中保存的 Git provenance：

```text
source_url
normalized_source
tracked_ref
installed_commit
```

检查更新流程：

1. 从数据库读取 `source_url`、`tracked_ref`、`installed_commit`。
2. 查询远端 ref 的最新 commit。
3. 与 `installed_commit` 比较。
4. 如果不同，标记为可更新。

执行更新流程：

1. clone 到临时 staging 目录。
2. 校验新目录包含合法 Skill 内容。
3. 替换 `~/.agent-forge/skills/{skill_id}`。
4. 更新 `installed_commit`。
5. 对干净的项目副本进行同步。
6. 对已修改的项目副本保持不覆盖，并标记需要用户处理。

目录名从 `skills` 变为 `anthropics-skills` 不应影响 Git 更新。

## 12. 旧数据迁移

如果历史数据中已存在使用 repo name 作为 `skill_id` 的 Git Skill，需要迁移为 `owner-repo`。

示例：

```text
skill_id          = skills
normalized_source = github.com/mattpocock/skills
new_skill_id      = mattpocock-skills
```

迁移内容：

- 全局目录：`~/.agent-forge/skills/skills` -> `~/.agent-forge/skills/mattpocock-skills`
- `skill_packages.skill_id`
- `skills_user_meta.skill_id`
- `project_skills.skill_id`
- `skill_descriptions.target_id`：包级记录从 `skills` 改为 `mattpocock-skills`；成员记录从 `skills::...` 改为 `mattpocock-skills::...`
- 项目副本目录：`<project>/.agents/skills/skills` -> `<project>/.agents/skills/mattpocock-skills`

迁移必须在事务中更新数据库。文件系统重命名与数据库更新需要有失败恢复策略，不能造成数据库指向不存在的目录。

如果目标目录已经存在，不能自动覆盖或合并。系统应报告迁移冲突，并保留原始目录和数据库记录。

## 13. 本地文件夹导入

本设计不改变本地文件夹导入。

本地导入仍可使用选择的文件夹名作为安装身份。只有 Git 导入强制使用 `owner-repo`。

原因是本地文件夹没有稳定的 `owner/repo` 来源结构，也没有远端更新身份。

## 14. 错误处理

系统必须处理以下错误：

- Git URL 无法归一化。
- `normalized_source` 少于三段，无法提取 `owner` 和 `repo`。
- 生成的 `skill_id` 为空。
- 目标目录已存在且无法生成可用后缀。
- staging clone 失败。
- 扫描不到合法 `SKILL.md`。
- 迁移时目标目录已存在。
- 迁移时项目副本被用户修改。

失败时不能覆盖已有 Skill，不能删除旧目录，不能写入半完成的数据库状态。

## 15. 测试要求

Rust 测试：

- `github.com/anthropics/skills` 生成 `anthropics-skills`。
- `github.com/mattpocock/skills` 生成 `mattpocock-skills`。
- 两个 repo name 都叫 `skills` 的仓库可以同时导入。
- HTTPS、SSH、`.git` 写法仍按同一个 `normalized_source` 去重。
- 导入预览返回 `install_id` 和 `normalized_source`。
- 更新使用 `source_url` 和 `tracked_ref`，不依赖目录名。
- 项目启用复制到 `.agents/skills/{skill_id}`。
- 旧 repo-name ID 可以迁移到 `owner-repo`。
- 迁移目标已存在时不覆盖。
- 成员 Skill ID 使用新的包级 `skill_id`。

前端测试：

- 导入弹窗显示安装 ID。
- 导入弹窗显示规范化 Git 来源。
- 重复来源时禁用确认导入。
- 非重复但 repo name 相同的仓库不显示重复警告。

## 16. 验收标准

- 用户可以同时导入 `anthropics/skills` 和 `mattpocock/skills`。
- 两个仓库在全局目录中分别为 `anthropics-skills` 和 `mattpocock-skills`。
- 两个仓库可以同时启用到同一个项目。
- 项目内目录分别为 `.agents/skills/anthropics-skills` 和 `.agents/skills/mattpocock-skills`。
- 两个仓库可以独立检查更新和执行更新。
- 同一个仓库的不同 Git URL 写法仍然判定为重复来源。
- 显示名继续使用 Skill 元数据，不依赖目录名。
- 旧数据迁移后，分类、备注、中文技能说明和项目启用关系不丢失。
