use crate::components::module_detail::*;
use crate::models::ModuleDetail;
use leptos::prelude::*;

#[component]
pub fn ModuleDetailView(
    module: ModuleDetail,
    on_close: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-4">
            // Conditional header based on context (modal vs page)
            {if let Some(close_fn) = on_close {
                view! {
                    <ModalHeader
                        title=module.title.clone()
                        id=module.id
                        version=module.version
                        credits=module.credits
                        languages=module.languages.clone()
                        moses_link=module.moses_link.clone()
                        on_close=close_fn
                    />
                }.into_any()
            } else {
                view! {
                    <PageHeader
                        title=module.title.clone()
                        id=module.id
                        version=module.version
                        credits=module.credits
                        languages=module.languages.clone()
                        moses_link=module.moses_link.clone()
                    />
                }.into_any()
            }}

            // Metadata section
            <MetadataSection
                faculty=module.faculty.clone()
                institute=module.institute.clone()
                fachgebiet=module.fachgebiet.clone()
                responsible_person=module.responsible_person.clone()
                examination_board=module.examination_board.clone()
                valid_since=module.valid_since.clone()
                valid_until=module.valid_until.clone()
            />

            // Description section
            <DescriptionSection
                learning_result=module.learning_result.clone()
                content=module.content.clone()
                teaching_information=module.teaching_information.clone()
            />

            // Components section
            <ComponentsSection components=module.components.clone() />

            // Exams section
            <ExamsSection exams=module.exams.clone() />

            // Workload section
            <WorkloadSection workload=module.workload.clone() />

            // Requirements section
            <RequirementsSection
                requirements=module.requirements.clone()
                registration=module.registration.clone()
            />

            // Administrative section
            <AdministrativeSection
                max_attendees=module.max_attendees
                duration=module.duration.clone()
                additional_info=module.additional_info.clone()
            />

            // Study programs section
            <ProgramsSection programs=module.study_programs.clone() />

            // Contact section
            <ContactSection contact=module.contact.clone() />
        </div>
    }
}
