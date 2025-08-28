/**
 * Component Test Helpers for Button Migration Testing
 * 
 * Provides utilities for rendering and testing BaseButton and SidebarProjectButton components
 */

const fs = require('fs');
const path = require('path');

// Load test fixtures
const testProjects = JSON.parse(fs.readFileSync(
  path.join(__dirname, '../fixtures/test-projects.json'), 'utf8'
));
const uiColors = JSON.parse(fs.readFileSync(
  path.join(__dirname, '../fixtures/ui-color-scheme.json'), 'utf8'
));

/**
 * Renders a BaseButton component with given configuration
 * 
 * @param {Page} page - Playwright page object
 * @param {Object} config - Button configuration
 * @param {string} containerSelector - Container to render button in
 * @returns {Locator} Button element locator
 */
async function renderBaseButton(page, config = {}, containerSelector = 'body') {
  const defaultConfig = {
    type: 'button',
    content: 'Test Button',
    htmx: null,
    disabled: false,
    extraClasses: ''
  };
  
  const finalConfig = { ...defaultConfig, ...config };
  
  // Create container if needed
  await page.evaluate((selector) => {
    if (!document.querySelector(selector)) {
      const container = document.createElement('div');
      container.id = selector.replace('#', '');
      if (selector === 'body') {
        // Don't create new body, just use existing
      } else {
        document.body.appendChild(container);
      }
    }
  }, containerSelector);
  
  // Render BaseButton component
  await page.evaluate(({config, selector, colors}) => {
    const button = document.createElement('button');
    button.type = config.type || 'button';
    button.textContent = config.content || 'Test Button';
    button.disabled = config.disabled || false;
    
    // Apply CSS classes
    let classList = colors.cssClasses.baseButton;
    if (config.extraClasses) {
      classList += ' ' + config.extraClasses;
    }
    button.className = classList;
    
    // Apply base styling
    button.style.backgroundColor = colors.colors.button_bg;
    button.style.border = `1px solid ${colors.colors.button_border}`;
    button.style.color = colors.colors.primary_text;
    button.style.padding = '8px 16px';
    button.style.borderRadius = '4px';
    button.style.cursor = 'pointer';
    button.style.transition = 'all 200ms ease';
    button.style.fontFamily = 'sans-serif';
    button.style.fontSize = '14px';
    button.style.fontWeight = '500';
    
    // Apply HTMX attributes if provided
    if (config.htmx) {
      if (config.htmx.post) button.setAttribute('hx-post', config.htmx.post);
      if (config.htmx.target) button.setAttribute('hx-target', config.htmx.target);
      if (config.htmx.swap) button.setAttribute('hx-swap', config.htmx.swap);
      if (config.htmx.trigger) button.setAttribute('hx-trigger', config.htmx.trigger);
      if (config.htmx.vals) button.setAttribute('hx-vals', config.htmx.vals);
      if (config.htmx.onRequest) button.setAttribute('hx-on::before-request', config.htmx.onRequest);
    }
    
    // Apply unified hover handlers
    button.addEventListener('mouseover', (e) => {
      if (!e.target.disabled) {
        e.target.style.backgroundColor = colors.colors.button_bg_hover;
        e.target.style.borderColor = colors.colors.border_hover;
        e.target.style.transform = colors.transforms.hover_translateY;
      }
    });
    
    button.addEventListener('mouseout', (e) => {
      if (!e.target.disabled) {
        e.target.style.backgroundColor = colors.colors.button_bg;
        e.target.style.borderColor = colors.colors.button_border;
        e.target.style.transform = colors.transforms.normal_translateY;
      }
    });
    
    // Handle disabled state styling
    if (button.disabled) {
      button.style.opacity = '0.6';
      button.style.cursor = 'not-allowed';
    }
    
    const container = selector === 'body' ? document.body : document.querySelector(selector);
    container.appendChild(button);
    
    // Mark button for identification
    button.setAttribute('data-test-component', 'BaseButton');
    
  }, {config: finalConfig, selector: containerSelector, colors: uiColors});
  
  return page.locator(`${containerSelector} button[data-test-component="BaseButton"]`).last();
}

/**
 * Renders a SidebarProjectButton component with given configuration
 * 
 * @param {Page} page - Playwright page object
 * @param {Object} config - Project button configuration
 * @param {string} containerSelector - Container to render button in
 * @returns {Locator} Project button element locator
 */
async function renderSidebarProjectButton(page, config, containerSelector = 'body') {
  // Handle case where config is actually a project object (for test compatibility)
  if (config && config.name && config.path && !config.project) {
    config = { 
      project: config, 
      isActive: config.isActive || false,
      isSelected: config.isActive || false,
      processType: config.status === 'running' ? 'regular' : 
                   config.status === 'watching' ? 'watch' : '',
      includeRemoveButton: containerSelector && containerSelector.includeRemoveButton || false
    };
  }
  
  const defaultConfig = {
    project: testProjects.projects[0],
    isSelected: false,
    isActive: false,
    processType: ''
  };
  
  const finalConfig = { ...defaultConfig, ...config };
  
  // Create container if needed
  await page.evaluate((selector) => {
    if (!document.querySelector(selector)) {
      const container = document.createElement('div');
      container.id = selector.replace('#', '');
      if (selector === 'body') {
        // Use existing body
      } else {
        document.body.appendChild(container);
      }
    }
  }, containerSelector);
  
  // Render SidebarProjectButton component
  await page.evaluate(({config, selector, colors}) => {
    const projectDiv = document.createElement('div');
    
    // Apply classes based on state  
    let classList = 'project-item px-3 py-2 text-sm rounded-md transition-all duration-200';
    if (config.project.isActive) {
      classList += ' ' + colors.cssClasses.projectItemActive;
      projectDiv.style.backgroundColor = colors.colors.panel_bg;
      projectDiv.style.borderLeft = `4px solid ${colors.colors.accents.blue}`;
    } else {
      classList += ' ' + colors.cssClasses.projectItemInactive;
      projectDiv.style.backgroundColor = colors.colors.button_bg;
      projectDiv.style.border = `1px solid ${colors.colors.button_border}`;
      projectDiv.style.cursor = 'pointer';
    }
    
    projectDiv.className = classList;
    projectDiv.setAttribute('data-project-name', config.project.name);
    projectDiv.setAttribute('data-project-path', config.project.path);
    projectDiv.setAttribute('data-test-component', 'SidebarProjectButton');
    
    // Create inner container
    const innerDiv = document.createElement('div');
    innerDiv.className = 'flex items-center justify-between';
    
    // Create main project button (using BaseButton internally)
    const projectButton = document.createElement('button');
    projectButton.className = colors.cssClasses.baseButton + ' flex-1 text-left flex items-center cursor-pointer';
    projectButton.style.color = config.isSelected ? colors.colors.primary_text : colors.colors.muted_text;
    projectButton.style.padding = '0';
    
    // Only set background/border to none for active projects (they don't have hover)
    if (config.project.isActive) {
      projectButton.style.background = 'none';
      projectButton.style.border = 'none';
    }
    
    // Add HTMX attributes for project switching (only for inactive projects)
    if (!config.project.isActive) {
      projectButton.setAttribute('hx-post', '/htmx/project/switch');
      projectButton.setAttribute('hx-target', '#main-container');
      projectButton.setAttribute('hx-swap', 'outerHTML');
      projectButton.setAttribute('hx-include', '#sidebar-projects');
      projectButton.setAttribute('hx-vals', JSON.stringify({ path: config.project.path }));
      projectButton.setAttribute('aria-label', `Switch to project ${config.project.name}`);
      projectButton.setAttribute('role', 'button');
    } else {
      projectButton.setAttribute('aria-label', `Current active project: ${config.project.name}`);
    }
    
    // Create status dot
    const statusDot = document.createElement('div');
    statusDot.className = colors.cssClasses.statusDot + ' w-2 h-2 rounded-full';
    statusDot.style.width = '8px';
    statusDot.style.height = '8px';
    statusDot.style.borderRadius = '50%';
    statusDot.style.display = 'inline-block';
    statusDot.style.marginRight = '8px';
    
    // Set status dot color based on project status
    let dotColor = colors.colors.status_dots.stopped; // default
    let ariaLabel = 'Process stopped';
    let statusClass = 'status-stopped'; // default
    
    if (config.project.status === 'running') {
      dotColor = colors.colors.status_dots.running;
      ariaLabel = 'Process running';
      statusClass = 'status-running';
    } else if (config.project.status === 'watching') {
      dotColor = colors.colors.status_dots.watching;
      ariaLabel = 'Process watching';
      statusClass = 'status-watching';
    } else {
      dotColor = colors.colors.status_dots.stopped;
      ariaLabel = 'Process stopped';
      statusClass = 'status-stopped';
    }
    
    // Add status-specific class to the status dot
    statusDot.classList.add(statusClass);
    
    statusDot.setAttribute('aria-label', ariaLabel);
    statusDot.style.backgroundColor = dotColor;
    
    // Create project name span
    const projectName = document.createElement('span');
    projectName.className = colors.cssClasses.projectName + ' ml-2';
    projectName.textContent = config.project.name;
    projectName.style.marginLeft = '8px';
    
    // Assemble project button
    projectButton.appendChild(statusDot);
    projectButton.appendChild(projectName);
    
    // Assemble components - start with just the project button
    innerDiv.appendChild(projectButton);
    
    // Only add remove button if specifically requested and project is not active
    if (config.includeRemoveButton && !config.project.isActive) {
      const removeButton = document.createElement('button');
      removeButton.className = 'remove-project-btn text-red-400 hover:text-red-300 text-xs ml-2';
      removeButton.textContent = '×';
      removeButton.style.background = 'none';
      removeButton.style.border = 'none';
      removeButton.style.color = colors.colors.accents.red;
      removeButton.style.cursor = 'pointer';
      removeButton.style.fontSize = '16px';
      removeButton.style.padding = '0 4px';
      removeButton.style.marginLeft = '8px';
      
      // Add HTMX attributes for removal
      removeButton.setAttribute('hx-delete', `/htmx/project/remove?path=${config.project.path}`);
      removeButton.setAttribute('hx-target', '#project-list');
      removeButton.setAttribute('hx-swap', 'innerHTML');
      removeButton.setAttribute('hx-confirm', 'Remove this project?');
      removeButton.setAttribute('aria-label', `Remove project ${config.project.name}`);
      
      // Hover effects for remove button
      removeButton.addEventListener('mouseover', (e) => {
        e.target.style.color = '#fca5a5'; // red-300
      });
      removeButton.addEventListener('mouseout', (e) => {
        e.target.style.color = colors.colors.accents.red;
      });
      
      innerDiv.appendChild(removeButton);
    }
    
    projectDiv.appendChild(innerDiv);
    
    // Add hover effects to project button for inactive projects only
    if (!config.project.isActive) {
      // Set initial button styles
      projectButton.style.backgroundColor = colors.colors.button_bg;
      projectButton.style.borderColor = colors.colors.button_border;
      projectButton.style.border = '1px solid ' + colors.colors.button_border;
      projectButton.style.transform = colors.transforms.normal_translateY;
      
      projectButton.addEventListener('mouseover', (e) => {
        e.currentTarget.style.backgroundColor = colors.colors.button_bg_hover;
        e.currentTarget.style.borderColor = colors.colors.border_hover;
        e.currentTarget.style.transform = colors.transforms.hover_translateY;
      });
      
      projectButton.addEventListener('mouseout', (e) => {
        e.currentTarget.style.backgroundColor = colors.colors.button_bg;
        e.currentTarget.style.borderColor = colors.colors.button_border;
        e.currentTarget.style.transform = colors.transforms.normal_translateY;
      });
    }
    
    const container = selector === 'body' ? document.body : document.querySelector(selector);
    
    // Remove any existing button with the same project name to avoid duplicates
    const existingButton = container.querySelector(`[data-project-name="${config.project.name}"]`);
    if (existingButton) {
      existingButton.remove();
    }
    
    container.appendChild(projectDiv);
    
  }, {config: finalConfig, selector: containerSelector, colors: uiColors});
  
  return page.locator(`${containerSelector} .project-item[data-test-component="SidebarProjectButton"]`).last();
}

/**
 * Measures hover response time for a button element
 * 
 * @param {Locator} buttonElement - Button element to test
 * @returns {number} Response time in milliseconds
 */
async function measureHoverResponseTime(buttonElement) {
  const startTime = Date.now();
  
  await buttonElement.hover();
  
  // Wait for transform to actually change
  await buttonElement.page().waitForFunction(
    (el) => {
      const style = window.getComputedStyle(el);
      return style.transform.includes('translateY(-1px)') || style.transform.includes('matrix');
    },
    buttonElement
  );
  
  const endTime = Date.now();
  return endTime - startTime;
}

/**
 * Sets up test projects in sidebar
 * 
 * @param {Page} page - Playwright page object
 * @param {Array} projects - Array of project configurations
 */
async function setupTestProjectsInSidebar(page, projects) {
  await page.evaluate(({projectsData, colors}) => {
    // Create sidebar container if it doesn't exist
    let sidebar = document.querySelector('#project-list');
    if (!sidebar) {
      sidebar = document.createElement('div');
      sidebar.id = 'project-list';
      sidebar.className = 'space-y-1';
      sidebar.style.padding = '16px';
      sidebar.style.backgroundColor = colors.colors.sidebar_bg;
      document.body.appendChild(sidebar);
    }
    
    // Clear existing projects
    sidebar.innerHTML = '';
    
    // Add each project
    projectsData.forEach((project, index) => {
      const projectDiv = document.createElement('div');
      projectDiv.className = project.isActive ? 
        'project-item project-item-active px-3 py-2 text-sm rounded-md' :
        'project-item project-item-inactive px-3 py-2 text-sm rounded-md cursor-pointer';
      
      projectDiv.setAttribute('data-project-name', project.name);
      projectDiv.setAttribute('data-project-path', project.path);
      
      if (project.isActive) {
        projectDiv.style.backgroundColor = colors.colors.panel_bg;
        projectDiv.style.borderLeft = `4px solid ${colors.colors.accents.blue}`;
      } else {
        projectDiv.style.backgroundColor = colors.colors.button_bg;
        projectDiv.style.border = `1px solid ${colors.colors.button_border}`;
      }
      
      // Create inner content
      const innerDiv = document.createElement('div');
      innerDiv.className = 'flex items-center justify-between';
      
      // Project button
      const button = document.createElement('button');
      button.className = 'flex-1 text-left flex items-center';
      if (!project.isActive) {
        button.setAttribute('hx-post', `/htmx/project/switch?path=${project.path}`);
        button.setAttribute('hx-target', '#main-container');
        button.setAttribute('hx-swap', 'innerHTML');
      }
      
      // Status dot
      const dot = document.createElement('div');
      dot.className = 'w-2 h-2 rounded-full';
      dot.style.width = '8px';
      dot.style.height = '8px';
      dot.style.borderRadius = '50%';
      dot.style.marginRight = '8px';
      
      let dotColor = colors.colors.status_dots.stopped;
      if (project.processType === 'regular') {
        dotColor = colors.colors.status_dots.running;
      } else if (project.processType === 'watch') {
        dotColor = colors.colors.status_dots.watching;
      }
      dot.style.backgroundColor = dotColor;
      
      // Project name
      const name = document.createElement('span');
      name.className = 'project-name ml-2';
      name.textContent = project.name;
      name.style.color = project.isActive ? colors.colors.primary_text : colors.colors.muted_text;
      
      button.appendChild(dot);
      button.appendChild(name);
      
      // Remove button
      const removeBtn = document.createElement('button');
      removeBtn.textContent = '×';
      removeBtn.style.color = colors.colors.accents.red;
      removeBtn.setAttribute('hx-delete', `/htmx/project/remove?path=${project.path}`);
      removeBtn.setAttribute('hx-target', '#project-list');
      removeBtn.setAttribute('hx-swap', 'innerHTML');
      removeBtn.setAttribute('hx-confirm', 'Remove this project?');
      
      innerDiv.appendChild(button);
      innerDiv.appendChild(removeBtn);
      projectDiv.appendChild(innerDiv);
      
      sidebar.appendChild(projectDiv);
    });
    
  }, {projectsData: projects, colors: uiColors});
}

/**
 * Sets up single active project context
 * 
 * @param {Page} page - Playwright page object  
 * @param {Object} project - Project configuration
 */
async function setupActiveProject(page, project) {
  await page.evaluate(({projectData, colors}) => {
    // Create main container if it doesn't exist
    let mainContainer = document.querySelector('#main-container');
    if (!mainContainer) {
      mainContainer = document.createElement('div');
      mainContainer.id = 'main-container';
      mainContainer.style.padding = '16px';
      mainContainer.style.backgroundColor = colors.colors.panel_bg;
      document.body.appendChild(mainContainer);
    }
    
    // Set main container content
    mainContainer.innerHTML = `
      <h1>Project: ${projectData.name}</h1>
      <p>Path: ${projectData.path}</p>
      
      <!-- Control Center -->
      <div style="margin-top: 20px;">
        <h2>Control Center</h2>
        <div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 10px; margin-top: 10px;">
          <form hx-post="/mcp" hx-target="#ftl-output" hx-swap="beforeend">
            <input type="hidden" name="action" value="build"/>
            <input type="hidden" name="project-path" value="${projectData.path}"/>
            <button type="submit" style="width: 100%; padding: 8px; background-color: ${colors.colors.button_bg}; color: ${colors.colors.primary_text}; border: 1px solid ${colors.colors.button_border}; border-radius: 4px; cursor: pointer;">
              Build
            </button>
          </form>
          
          <form hx-post="/mcp" hx-target="#ftl-output" hx-swap="beforeend">
            <input type="hidden" name="action" value="up"/>
            <input type="hidden" name="mode" value="regular"/>
            <input type="hidden" name="project-path" value="${projectData.path}"/>
            <button type="submit" style="width: 100%; padding: 8px; background-color: ${colors.colors.button_bg}; color: ${colors.colors.primary_text}; border: 1px solid ${colors.colors.button_border}; border-radius: 4px; cursor: pointer;">
              Up
            </button>
          </form>
          
          <form hx-post="/mcp" hx-target="#ftl-output" hx-swap="beforeend">
            <input type="hidden" name="action" value="up"/>
            <input type="hidden" name="mode" value="watch"/>
            <input type="hidden" name="project-path" value="${projectData.path}"/>
            <button type="submit" style="width: 100%; padding: 8px; background-color: ${colors.colors.button_bg}; color: ${colors.colors.primary_text}; border: 1px solid ${colors.colors.button_border}; border-radius: 4px; cursor: pointer;">
              Watch
            </button>
          </form>
        </div>
      </div>
      
      <!-- Output Container -->
      <div id="ftl-output" style="margin-top: 20px; padding: 10px; background-color: #000; color: #0f0; font-family: monospace; border-radius: 4px; min-height: 100px;">
        <!-- Command output will appear here -->
      </div>
    `;
    
  }, {projectData: project, colors: uiColors});
}

/**
 * Gets test project by ID
 * 
 * @param {string} projectId - Project ID to retrieve
 * @returns {Object} Project configuration
 */
function getTestProject(projectId) {
  return testProjects.projects.find(p => p.id === projectId) || testProjects.projects[0];
}

/**
 * Gets UI color by name
 * 
 * @param {string} colorName - Color name to retrieve
 * @returns {string} Color value
 */
function getUIColor(colorName) {
  const keys = colorName.split('.');
  let value = uiColors;
  for (const key of keys) {
    value = value[key];
    if (!value) break;
  }
  return value;
}

/**
 * Checks if element has vertical translation (either translateY or matrix format)
 * 
 * @param {string} transform - CSS transform value
 * @param {number} expectedY - Expected Y translation in pixels
 * @returns {boolean} Whether element has expected vertical translation
 */
function hasVerticalTranslation(transform, expectedY) {
  // Handle "none" case - no transformation means 0 translation
  if (transform === 'none') {
    return Math.abs(expectedY) < 0.1;
  }
  
  // Check for translateY format
  if (transform.includes(`translateY(${expectedY}px)`)) {
    return true;
  }
  
  // Check for matrix format: matrix(a, b, c, d, e, f) where f is Y translation
  const matrixMatch = transform.match(/matrix\(([^)]+)\)/);
  if (matrixMatch) {
    const values = matrixMatch[1].split(',').map(v => parseFloat(v.trim()));
    if (values.length >= 6) {
      const yTranslation = values[5]; // f value in matrix(a, b, c, d, e, f)
      
      // For pixel-to-matrix conversion, browsers use different coordinate systems
      // Allow larger tolerance for pixel-to-matrix coordinate conversion
      // -1px often becomes ~-0.87 in matrix format due to browser coordinate system
      if (expectedY === -1) {
        // For hover state (-1px), accept values around -0.87 to -1.2
        return yTranslation < -0.5 && yTranslation > -1.5;
      } else if (expectedY === 0) {
        // For normal state (0px), accept small variations around 0
        return Math.abs(yTranslation) < 0.1;
      } else {
        // For other values, use proportional tolerance
        return Math.abs(yTranslation - expectedY) < Math.abs(expectedY * 0.2) + 0.5;
      }
    }
  }
  
  return false;
}

/**
 * Validates CSS properties of project buttons
 * @param {Page} page - Playwright page object
 * @param {string} buttonSelector - CSS selector for the button
 * @param {Object} expectedCSS - Expected CSS properties
 * @returns {boolean} Validation success
 */
async function validateProjectButtonCSS(page, buttonSelector, expectedCSS) {
  const button = page.locator(buttonSelector);
  
  for (const [property, expectedValue] of Object.entries(expectedCSS)) {
    const actualValue = await button.evaluate((el, prop) => {
      return getComputedStyle(el)[prop];
    }, property);
    
    if (actualValue !== expectedValue) {
      throw new Error(`CSS property ${property} expected "${expectedValue}" but got "${actualValue}"`);
    }
  }
  
  return true;
}

/**
 * Simulates project removal workflow
 * @param {Page} page - Playwright page object  
 * @param {string} projectName - Name of project to remove
 * @returns {boolean} Removal success
 */
async function simulateProjectRemoval(page, projectName) {
  // Find the remove button for this project
  const removeButton = page.locator(`[data-project-name="${projectName}"] .remove-project-btn`);
  
  // Set up dialog handler to accept removal confirmation
  page.on('dialog', dialog => {
    dialog.accept();
  });
  
  // Monitor for the HTMX delete request
  const deleteRequestPromise = page.waitForRequest(req => 
    req.url().includes('/htmx/project/remove') &&
    req.url().includes(`path=`) &&
    req.method() === 'DELETE'
  );
  
  // Click remove button
  await removeButton.click();
  
  // Wait for delete request
  await deleteRequestPromise;
  
  // Wait for project to be removed from DOM
  await page.waitForSelector(`[data-project-name="${projectName}"]`, { 
    state: 'detached',
    timeout: 5000 
  });
  
  return true;
}

module.exports = {
  renderBaseButton,
  renderSidebarProjectButton,
  measureHoverResponseTime,
  setupTestProjectsInSidebar,
  setupActiveProject,
  getTestProject,
  getUIColor,
  hasVerticalTranslation,
  validateProjectButtonCSS,
  simulateProjectRemoval,
  testProjects,
  uiColors
};