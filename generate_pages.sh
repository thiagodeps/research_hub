#!/bin/bash
declare -A pages
pages=(
  ["students"]="name"
  ["campuses"]="name"
  ["organizations"]="name"
  ["fellowships"]="name"
  ["proficiencies"]="name"
  ["professional_activities"]="name"
  ["knowledge_areas"]="name"
  ["languages"]="name"
  ["research_productions"]="title"
)

for entity in "${!pages[@]}"; do
  prop="${pages[$entity]}"
  label="Nome"
  if [ "$prop" = "title" ]; then
    label="Título"
  fi
  cat << ASTRO > frontend/src/pages/dashboard/${entity}.astro
---
import Dashboard from '../../layouts/Dashboard.astro';
import EntityPage from '../../components/EntityPage.jsx';
---

<Dashboard>
  <EntityPage 
    client:load 
    entity="$entity" 
    columns={['id', '$prop']} 
    fields={[
      { name: 'id', label: 'ID', type: 'number' },
      { name: '$prop', label: '$label', required: true }
    ]} 
  />
</Dashboard>
ASTRO
done
