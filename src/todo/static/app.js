/**
 * TaskMaster - Frontend Application Logic
 */

document.addEventListener('DOMContentLoaded', () => {
  // State
  let currentFilter = 'all'; // 'all' | 'active' | 'completed'
  let currentPriorityFilter = null; // null | 'high' | 'medium' | 'low'
  let searchQuery = '';
  let searchDebounceTimeout = null;

  // DOM Elements
  const themeToggleBtn = document.getElementById('themeToggleBtn');
  const currentDateEl = document.getElementById('currentDate');
  const progressStatsText = document.getElementById('progressStatsText');
  const progressBarFill = document.getElementById('progressBarFill');
  const highCountEl = document.getElementById('highCount');
  const medCountEl = document.getElementById('medCount');
  const lowCountEl = document.getElementById('lowCount');
  
  const addTodoForm = document.getElementById('addTodoForm');
  const todoTitleInput = document.getElementById('todoTitleInput');
  const toggleDetailsBtn = document.getElementById('toggleDetailsBtn');
  const detailsCollapsible = document.getElementById('detailsCollapsible');
  const todoDescInput = document.getElementById('todoDescInput');
  
  const filterTabs = document.querySelectorAll('.tab-btn');
  const searchInput = document.getElementById('searchInput');
  const clearSearchBtn = document.getElementById('clearSearchBtn');
  const activePriorityBadge = document.getElementById('activePriorityBadge');
  const filterPriorityName = document.getElementById('filterPriorityName');
  const clearPriorityFilterBtn = document.getElementById('clearPriorityFilterBtn');
  const priorityPills = document.querySelectorAll('.pill');

  const todoList = document.getElementById('todoList');
  const emptyState = document.getElementById('emptyState');
  const emptyTitle = document.getElementById('emptyTitle');
  const emptySubtitle = document.getElementById('emptySubtitle');
  const itemsLeftCount = document.getElementById('itemsLeftCount');
  const clearCompletedBtn = document.getElementById('clearCompletedBtn');

  const editModal = document.getElementById('editModal');
  const editTodoForm = document.getElementById('editTodoForm');
  const editTodoId = document.getElementById('editTodoId');
  const editTodoTitle = document.getElementById('editTodoTitle');
  const editTodoDesc = document.getElementById('editTodoDesc');
  const closeEditModalBtn = document.getElementById('closeEditModalBtn');
  const cancelEditBtn = document.getElementById('cancelEditBtn');

  const toastContainer = document.getElementById('toastContainer');

  // Initialize
  initTheme();
  initDate();
  loadTodos();
  loadStats();

  // ---------------- Theme Management ----------------
  function initTheme() {
    const savedTheme = localStorage.getItem('taskmaster_theme');
    if (savedTheme) {
      document.documentElement.setAttribute('data-theme', savedTheme);
    } else if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
      document.documentElement.setAttribute('data-theme', 'dark');
    }

    themeToggleBtn.addEventListener('click', () => {
      const current = document.documentElement.getAttribute('data-theme');
      const next = current === 'dark' ? 'light' : 'dark';
      document.documentElement.setAttribute('data-theme', next);
      localStorage.setItem('taskmaster_theme', next);
    });
  }

  function initDate() {
    const now = new Date();
    const options = { weekday: 'long', month: 'short', day: 'numeric', year: 'numeric' };
    currentDateEl.textContent = now.toLocaleDateString(undefined, options);
  }

  // ---------------- Details toggle ----------------
  toggleDetailsBtn.addEventListener('click', () => {
    detailsCollapsible.classList.toggle('collapsed');
    if (!detailsCollapsible.classList.contains('collapsed')) {
      todoDescInput.focus();
    }
  });

  // ---------------- Filter tabs ----------------
  filterTabs.forEach((tab) => {
    tab.addEventListener('click', () => {
      filterTabs.forEach((t) => t.classList.remove('active'));
      tab.classList.add('active');
      currentFilter = tab.dataset.filter;
      loadTodos();
    });
  });

  // ---------------- Priority Pill Filters ----------------
  priorityPills.forEach((pill) => {
    pill.addEventListener('click', () => {
      const priority = pill.dataset.priority;
      if (currentPriorityFilter === priority) {
        clearPriorityFilter();
      } else {
        currentPriorityFilter = priority;
        activePriorityBadge.style.display = 'flex';
        filterPriorityName.textContent = priority.toUpperCase();
        loadTodos();
      }
    });
  });

  clearPriorityFilterBtn.addEventListener('click', clearPriorityFilter);

  function clearPriorityFilter() {
    currentPriorityFilter = null;
    activePriorityBadge.style.display = 'none';
    loadTodos();
  }

  // ---------------- Search ----------------
  searchInput.addEventListener('input', (e) => {
    searchQuery = e.target.value.trim();
    clearSearchBtn.style.display = searchQuery ? 'block' : 'none';

    clearTimeout(searchDebounceTimeout);
    searchDebounceTimeout = setTimeout(() => {
      loadTodos();
    }, 250);
  });

  clearSearchBtn.addEventListener('click', () => {
    searchInput.value = '';
    searchQuery = '';
    clearSearchBtn.style.display = 'none';
    loadTodos();
  });

  // ---------------- Add Todo ----------------
  addTodoForm.addEventListener('submit', async (e) => {
    e.preventDefault();
    const title = todoTitleInput.value.trim();
    if (!title) return;

    const desc = todoDescInput.value.trim();
    const priority = document.querySelector('input[name="priority"]:checked')?.value || 'medium';

    try {
      const res = await fetch('/api/todos', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ title, description: desc, priority }),
      });

      if (!res.ok) {
        const err = await res.json();
        showToast(err.detail || 'Failed to add task', 'error');
        return;
      }

      todoTitleInput.value = '';
      todoDescInput.value = '';
      detailsCollapsible.classList.add('collapsed');
      document.querySelector('input[name="priority"][value="medium"]').checked = true;

      showToast('Task added successfully!');
      loadTodos();
      loadStats();
    } catch (err) {
      showToast('Network error while adding task', 'error');
    }
  });

  // ---------------- Clear Completed ----------------
  clearCompletedBtn.addEventListener('click', async () => {
    try {
      const res = await fetch('/api/todos-completed/clear', {
        method: 'DELETE',
      });
      const data = await res.json();
      if (data.cleared_count > 0) {
        showToast(`Cleared ${data.cleared_count} completed task${data.cleared_count > 1 ? 's' : ''}`);
        loadTodos();
        loadStats();
      } else {
        showToast('No completed tasks to clear');
      }
    } catch (err) {
      showToast('Failed to clear completed tasks', 'error');
    }
  });

  // ---------------- Edit Modal ----------------
  function openEditModal(todo) {
    editTodoId.value = todo.id;
    editTodoTitle.value = todo.title;
    editTodoDesc.value = todo.description || '';
    const radio = document.querySelector(`input[name="editPriority"][value="${todo.priority}"]`);
    if (radio) radio.checked = true;

    editModal.style.display = 'flex';
    editTodoTitle.focus();
  }

  function closeEditModal() {
    editModal.style.display = 'none';
    editTodoForm.reset();
  }

  closeEditModalBtn.addEventListener('click', closeEditModal);
  cancelEditBtn.addEventListener('click', closeEditModal);

  editModal.addEventListener('click', (e) => {
    if (e.target === editModal) closeEditModal();
  });

  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && editModal.style.display === 'flex') {
      closeEditModal();
    }
  });

  editTodoForm.addEventListener('submit', async (e) => {
    e.preventDefault();
    const id = editTodoId.value;
    const title = editTodoTitle.value.trim();
    const description = editTodoDesc.value.trim();
    const priority = document.querySelector('input[name="editPriority"]:checked')?.value || 'medium';

    if (!title) return;

    try {
      const res = await fetch(`/api/todos/${id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ title, description, priority }),
      });

      if (!res.ok) {
        const err = await res.json();
        showToast(err.detail || 'Failed to update task', 'error');
        return;
      }

      closeEditModal();
      showToast('Task updated successfully');
      loadTodos();
      loadStats();
    } catch (err) {
      showToast('Network error while updating task', 'error');
    }
  });

  // ---------------- Fetch Todos & Stats ----------------
  async function loadTodos() {
    const params = new URLSearchParams();
    if (currentFilter !== 'all') params.append('status', currentFilter);
    if (currentPriorityFilter) params.append('priority', currentPriorityFilter);
    if (searchQuery) params.append('search', searchQuery);

    try {
      const res = await fetch(`/api/todos?${params.toString()}`);
      if (!res.ok) throw new Error('Failed to fetch tasks');
      const todos = await res.json();
      renderTodoList(todos);
    } catch (err) {
      console.error(err);
      showToast('Error loading tasks', 'error');
    }
  }

  async function loadStats() {
    try {
      const res = await fetch('/api/stats');
      if (!res.ok) return;
      const stats = await res.json();

      // Update progress
      progressBarFill.style.width = `${stats.completion_percentage}%`;
      progressStatsText.textContent = `${stats.completed} / ${stats.total} completed (${stats.completion_percentage}%)`;
      itemsLeftCount.textContent = `${stats.active} item${stats.active === 1 ? '' : 's'} left`;

      // Update priority counts
      highCountEl.textContent = stats.by_priority.high || 0;
      medCountEl.textContent = stats.by_priority.medium || 0;
      lowCountEl.textContent = stats.by_priority.low || 0;

      // Disable clear completed if 0 completed
      clearCompletedBtn.disabled = stats.completed === 0;
    } catch (err) {
      console.error(err);
    }
  }

  // ---------------- Render List ----------------
  function renderTodoList(todos) {
    todoList.innerHTML = '';

    if (todos.length === 0) {
      emptyState.style.display = 'flex';
      if (searchQuery) {
        emptyTitle.textContent = 'No matching tasks';
        emptySubtitle.textContent = `No tasks matched your search "${searchQuery}".`;
      } else if (currentPriorityFilter) {
        emptyTitle.textContent = `No ${currentPriorityFilter} priority tasks`;
        emptySubtitle.textContent = `There are no tasks with ${currentPriorityFilter} priority.`;
      } else if (currentFilter === 'completed') {
        emptyTitle.textContent = 'No completed tasks yet';
        emptySubtitle.textContent = 'Complete some tasks to see them listed here.';
      } else if (currentFilter === 'active') {
        emptyTitle.textContent = 'All caught up!';
        emptySubtitle.textContent = 'You have no pending tasks. Great job!';
      } else {
        emptyTitle.textContent = 'No tasks yet';
        emptySubtitle.textContent = 'Add your first task above to get started!';
      }
      return;
    }

    emptyState.style.display = 'none';

    todos.forEach((todo) => {
      const li = document.createElement('li');
      li.className = `todo-item ${todo.completed ? 'completed' : ''}`;
      li.dataset.id = todo.id;

      // Checkbox
      const checkWrapper = document.createElement('div');
      checkWrapper.className = 'todo-checkbox-wrapper';
      const checkbox = document.createElement('input');
      checkbox.type = 'checkbox';
      checkbox.className = 'custom-checkbox';
      checkbox.checked = todo.completed;
      checkbox.setAttribute('aria-label', `Mark "${todo.title}" as ${todo.completed ? 'incomplete' : 'complete'}`);
      checkbox.addEventListener('change', () => toggleTodo(todo.id));
      checkWrapper.appendChild(checkbox);

      // Content
      const content = document.createElement('div');
      content.className = 'todo-content';

      const title = document.createElement('div');
      title.className = 'todo-title';
      title.textContent = todo.title;
      content.appendChild(title);

      if (todo.description) {
        const desc = document.createElement('div');
        desc.className = 'todo-desc';
        desc.textContent = todo.description;
        content.appendChild(desc);
      }

      // Meta
      const meta = document.createElement('div');
      meta.className = 'todo-meta';

      const priorityTag = document.createElement('span');
      priorityTag.className = `priority-tag ${todo.priority}-tag`;
      priorityTag.textContent = todo.priority.toUpperCase();
      meta.appendChild(priorityTag);

      const timeTag = document.createElement('span');
      timeTag.className = 'todo-time';
      timeTag.textContent = formatDate(todo.created_at);
      meta.appendChild(timeTag);

      content.appendChild(meta);

      // Actions
      const actions = document.createElement('div');
      actions.className = 'todo-actions';

      // Edit Button
      const editBtn = document.createElement('button');
      editBtn.className = 'action-btn edit-btn';
      editBtn.title = 'Edit task';
      editBtn.setAttribute('aria-label', 'Edit task');
      editBtn.innerHTML = `
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 20h9"></path>
          <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"></path>
        </svg>
      `;
      editBtn.addEventListener('click', () => openEditModal(todo));

      // Delete Button
      const delBtn = document.createElement('button');
      delBtn.className = 'action-btn delete-btn';
      delBtn.title = 'Delete task';
      delBtn.setAttribute('aria-label', 'Delete task');
      delBtn.innerHTML = `
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="3 6 5 6 21 6"></polyline>
          <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
        </svg>
      `;
      delBtn.addEventListener('click', () => deleteTodo(todo.id));

      actions.appendChild(editBtn);
      actions.appendChild(delBtn);

      li.appendChild(checkWrapper);
      li.appendChild(content);
      li.appendChild(actions);

      todoList.appendChild(li);
    });
  }

  // ---------------- Actions ----------------
  async function toggleTodo(id) {
    try {
      const res = await fetch(`/api/todos/${id}/toggle`, { method: 'PATCH' });
      if (!res.ok) throw new Error();
      loadTodos();
      loadStats();
    } catch (err) {
      showToast('Failed to update task state', 'error');
    }
  }

  async function deleteTodo(id) {
    try {
      const res = await fetch(`/api/todos/${id}`, { method: 'DELETE' });
      if (!res.ok) throw new Error();
      showToast('Task deleted');
      loadTodos();
      loadStats();
    } catch (err) {
      showToast('Failed to delete task', 'error');
    }
  }

  function formatDate(isoStr) {
    if (!isoStr) return '';
    try {
      const d = new Date(isoStr);
      return d.toLocaleDateString(undefined, {
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
      });
    } catch {
      return '';
    }
  }

  // ---------------- Toast notification ----------------
  function showToast(message, type = 'info') {
    const toast = document.createElement('div');
    toast.className = `toast ${type}`;
    toast.textContent = message;

    toastContainer.appendChild(toast);

    setTimeout(() => {
      toast.style.transition = 'opacity 0.3s ease, transform 0.3s ease';
      toast.style.opacity = '0';
      toast.style.transform = 'translateY(10px)';
      setTimeout(() => toast.remove(), 300);
    }, 2600);
  }
});
