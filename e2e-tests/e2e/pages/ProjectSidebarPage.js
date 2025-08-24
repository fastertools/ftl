// Page Object for the project sidebar
const HTMXHelpers = require('../utils/HTMXHelpers');

class ProjectSidebarPage {
    constructor(page) {
        this.page = page;
        
        // Selectors
        this.addProjectButton = 'button:has-text("Add Project")';
        this.projectNameInput = 'input[name="name"]';
        this.projectPathInput = 'input[name="path"]';
        this.addButton = 'button[type="submit"]:has-text("Add")';
        this.cancelButton = 'button:has-text("Cancel")';
        this.projectList = '#project-list';
        this.removeProjectButton = 'button.text-red-400';
    }

    async clickAddProject() {
        const button = await this.page.locator(this.addProjectButton).first();
        await button.click();
        await HTMXHelpers.waitForSettle(this.page, { 
            timeout: 1000,
            projectPath: this.page.context().projectPath 
        });
    }

    async fillProjectForm(name, path) {
        await this.page.fill(this.projectNameInput, name);
        await this.page.fill(this.projectPathInput, path);
    }

    async submitProjectForm() {
        await this.page.locator(this.addButton).click();
        await HTMXHelpers.waitForSettle(this.page, { 
            timeout: 1000,
            projectPath: this.page.context().projectPath 
        });
    }

    async cancelProjectForm() {
        await this.page.locator(this.cancelButton).click();
        await HTMXHelpers.waitForSettle(this.page, { 
            timeout: 1000,
            projectPath: this.page.context().projectPath 
        });
    }

    async getProjectCount() {
        // Count only actual project status dots (with bg-color, not border)
        // Project dots have bg-green-500 or bg-gray-500, Add Project has border-gray-500
        const projectStatusDots = await this.page.locator(`${this.projectList} span.bg-green-500, ${this.projectList} span.bg-gray-500`).all();
        return projectStatusDots.length;
    }

    async projectExists(name) {
        // Only look for projects in the sidebar project list, not in the main content area
        // Look for project names in spans within the project list structure, but exclude Add Project
        const project = await this.page.locator(`${this.projectList} span:has-text("${name}"):not(:has-text("Add Project"))`).first();
        return await project.count() > 0;
    }

    async switchToProject(projectName) {
        const project = await this.page.locator(`div:has-text("${projectName}")`).first();
        if (await project.count() > 0) {
            await project.click();
            // Wait for potential page refresh
            await this.page.waitForLoadState('domcontentloaded');
        }
    }

    async removeProject(projectName) {
        const project = await this.page.locator(`div:has-text("${projectName}")`).first();
        if (await project.count() > 0) {
            const removeBtn = await project.locator(this.removeProjectButton).first();
            if (await removeBtn.count() > 0) {
                // Handle confirmation dialog if present
                this.page.once('dialog', async dialog => await dialog.accept());
                await removeBtn.click();
                await HTMXHelpers.waitForSettle(this.page, { 
                    timeout: 1000,
                    projectPath: this.page.context().projectPath 
                });
            }
        }
    }

    async isFormVisible() {
        const form = await this.page.locator('form').first();
        return await form.count() > 0;
    }

    async isAddButtonVisible() {
        const button = await this.page.locator(this.addProjectButton).first();
        return await button.count() > 0;
    }
}

module.exports = ProjectSidebarPage;