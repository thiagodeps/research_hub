import re
import os

content = open('backend/schema_dump.txt').read()
tables = {}
current_table = None

for line in content.split('\n'):
    line = line.strip()
    if not line: continue
    m = re.match(r'--- (.*)_canonical.parquet ---', line)
    if m:
        current_table = m.group(1)
        tables[current_table] = []
        continue
    
    if current_table and 'dtype:' not in line and line != '--':
        parts = line.split()
        if len(parts) >= 2:
            col_name = parts[0]
            tables[current_table].append(col_name)

class_map = {
    "Researcher": ("researchers", "name"),
    "Article": ("articles", "title"),
    "ResearchGroup": ("groups", "name"),
    "Initiative": ("initiatives", "name"),
    "Advisorship": ("advisorships", "name"),
    "Award": ("awards", "title"),
    "Student": ("students", "name"),
    "Campus": ("campuses", "name"),
    "Organization": ("organizations", "name"),
    "Fellowship": ("fellowships", "name"),
    "Proficiency": ("proficiencies", "name"),
    "ProfessionalActivity": ("professional_activities", "name"),
    "KnowledgeArea": ("knowledge_areas", "name"),
    "Language": ("languages", "name"),
    "ResearchProduction": ("research_productions", "title")
}

# The columns we want to show as json_readonly
json_cols = [
    'initiatives', 'research_groups', 'knowledge_areas', 'academic_education', 
    'articles', 'advisorships', 'organization', 'campus', 'team', 'students', 'researchers'
]

for cls, (route_name, main_col) in class_map.items():
    # Find matching parquet table name
    # e.g. route_name "groups" -> "research_groups"
    pq_name = route_name
    if route_name == "groups": pq_name = "research_groups"
    
    table_cols = tables.get(pq_name, [])
    
    display_cols = ['id', main_col]
    fields = [
        f"{{ name: 'id', label: 'ID', type: 'number' }}",
        f"{{ name: '{main_col}', label: '{main_col.capitalize()}', required: true }}"
    ]
    
    for c in table_cols:
        if c in json_cols and c not in display_cols:
            display_cols.append(c)
            fields.append(f"{{ name: '{c}', label: '{c.capitalize()} (Vínculos)', type: 'json_readonly' }}")
    
    columns_str = "[" + ", ".join([f"'{c}'" for c in display_cols]) + "]"
    fields_str = "[\n      " + ",\n      ".join(fields) + "\n    ]"
    
    astro_content = f"""---
import Dashboard from '../../layouts/Dashboard.astro';
import EntityPage from '../../components/EntityPage.jsx';
---

<Dashboard>
  <EntityPage 
    client:load 
    entity="{route_name}" 
    columns={{{columns_str}}} 
    fields={{{fields_str}}} 
  />
</Dashboard>
"""
    with open(f"frontend/src/pages/dashboard/{route_name}.astro", "w") as f:
        f.write(astro_content)

print("Astro pages updated!")
