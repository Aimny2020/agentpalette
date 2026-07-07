import type { Skill, SkillMember } from '../../shared/api/types';

export type CatalogResult =
  | { type: 'skill'; skill: Skill }
  | { type: 'member'; skill: Skill; member: SkillMember };

export function projectCatalog(
  skills: Skill[],
  search: string,
  selectedCategoryId: string | null = null,
): CatalogResult[] {
  const query = search.trim().toLocaleLowerCase();
  const categoryMatches = (skill: Skill) => {
    if (selectedCategoryId === 'uncategorized') return !skill.category_id;
    if (selectedCategoryId) return skill.category_id === selectedCategoryId;
    return true;
  };

  return skills.flatMap((skill): CatalogResult[] => {
    if (!categoryMatches(skill)) return [];
    if (!query) return [{ type: 'skill', skill }];

    const customDesc = skill.custom_description || '';
    const parentText = `${skill.metadata.name} ${skill.metadata.description} ${customDesc}`.toLocaleLowerCase();
    if (parentText.includes(query)) return [{ type: 'skill', skill }];

    return skill.members
      .filter((member) => {
        const memberCustomDesc = member.custom_description || '';
        return `${member.metadata.name} ${member.metadata.description} ${memberCustomDesc}`
          .toLocaleLowerCase()
          .includes(query);
      })
      .map((member) => ({ type: 'member' as const, skill, member }));
  });
}
