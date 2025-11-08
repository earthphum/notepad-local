// ===== API Configuration =====
const API_BASE_URL = "http://localhost:3000";
let authToken = localStorage.getItem("authToken");
let currentUser = null;

// ===== DOM Elements =====
const elements = {
  // Views
  publicView: document.getElementById("publicView"),
  privateView: document.getElementById("privateView"),
  statsContainer: document.getElementById("statsContainer"),

  // Navigation
  navBtns: document.querySelectorAll(".nav-btn"),
  loginBtn: document.getElementById("loginBtn"),
  logoutBtn: document.getElementById("logoutBtn"),
  userInfo: document.getElementById("userInfo"),
  username: document.getElementById("username"),

  // Notes
  publicNotesGrid: document.getElementById("publicNotesGrid"),
  privateNotesGrid: document.getElementById("privateNotesGrid"),

  // Modals
  noteModal: document.getElementById("noteModal"),
  loginModal: document.getElementById("loginModal"),
  detailModal: document.getElementById("detailModal"),

  // Forms
  noteForm: document.getElementById("noteForm"),
  loginForm: document.getElementById("loginForm"),

  // Modal Elements
  modalTitle: document.getElementById("modalTitle"),
  noteTitle: document.getElementById("noteTitle"),
  noteContent: document.getElementById("noteContent"),
  notePublic: document.getElementById("notePublic"),
  submitBtnText: document.getElementById("submitBtnText"),
  cancelBtn: document.getElementById("cancelBtn"),

  // Detail Modal
  detailTitle: document.getElementById("detailTitle"),
  detailAuthor: document.getElementById("detailAuthor"),
  detailDate: document.getElementById("detailDate"),
  detailVisibility: document.getElementById("detailVisibility"),
  detailContent: document.getElementById("detailContent"),
  noteActions: document.getElementById("noteActions"),

  // Stats
  totalNotes: document.getElementById("totalNotes"),
  publicNotes: document.getElementById("publicNotes"),
  privateNotes: document.getElementById("privateNotes"),

  // Login
  loginUsername: document.getElementById("username"),
  loginPassword: document.getElementById("password"),

  // Toast
  toast: document.getElementById("toast"),
  toastIcon: document.getElementById("toastIcon"),
  toastMessage: document.getElementById("toastMessage"),
  toastProgress: document.getElementById("toastProgress"),

  // Close buttons
  closeModal: document.getElementById("closeModal"),
  closeLoginModal: document.getElementById("closeLoginModal"),
  closeDetailModal: document.getElementById("closeDetailModal"),

  // Action buttons
  createNoteBtn: document.getElementById("createNoteBtn"),
};

// ===== State Management =====
let currentView = "public";
let editingNoteId = null;
let notesCache = {
  public: [],
  private: [],
};

// ===== Utility Functions =====
function formatDate(dateString) {
  const date = new Date(dateString);
  const now = new Date();
  const diff = now - date;
  const hours = Math.floor(diff / (1000 * 60 * 60));
  const days = Math.floor(hours / 24);

  if (days > 0) {
    return `${days} day${days > 1 ? "s" : ""} ago`;
  } else if (hours > 0) {
    return `${hours} hour${hours > 1 ? "s" : ""} ago`;
  } else {
    return "Just now";
  }
}

function truncateText(text, maxLength = 150) {
  if (text.length <= maxLength) return text;
  return text.substring(0, maxLength).trim() + "...";
}

function escapeHtml(text) {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
}

function showToast(message, type = "success") {
  elements.toastMessage.textContent = message;
  elements.toastIcon.className = `toast-icon fas ${type === "success" ? "fa-check-circle" : type === "error" ? "fa-exclamation-circle" : "fa-info-circle"}`;
  elements.toast.className = `toast ${type}`;

  // Reset and restart progress animation
  elements.toastProgress.style.animation = "none";
  void elements.toastProgress.offsetHeight; // Trigger reflow
  elements.toastProgress.style.animation = "progress 3s linear forwards";

  elements.toast.classList.add("show");

  setTimeout(() => {
    elements.toast.classList.remove("show");
  }, 3000);
}

// ===== API Functions =====
async function apiRequest(endpoint, options = {}) {
  const url = `${API_BASE_URL}${endpoint}`;
  const config = {
    headers: {
      "Content-Type": "application/json",
      ...options.headers,
    },
    ...options,
  };

  if (authToken) {
    config.headers.Authorization = `Bearer ${authToken}`;
  }

  try {
    const response = await fetch(url, config);
    const data = await response.json();

    if (!response.ok) {
      throw new Error(data.error || `HTTP ${response.status}`);
    }

    return data;
  } catch (error) {
    console.error("API Error:", error);
    showToast(error.message || "Request failed", "error");
    throw error;
  }
}

async function login(username, password) {
  try {
    const data = await apiRequest("/login", {
      method: "POST",
      body: JSON.stringify({ username, password }),
    });

    authToken = data.token;
    currentUser = username;
    localStorage.setItem("authToken", authToken);
    localStorage.setItem("currentUser", username);

    updateAuthUI();
    showToast("Login successful!", "success");

    if (currentView === "private") {
      await loadPrivateNotes();
    } else if (currentView === "stats") {
      await loadStats();
    }

    elements.loginModal.classList.remove("active");
    return true;
  } catch (error) {
    showToast("Login failed: " + error.message, "error");
    return false;
  }
}

async function logout() {
  try {
    authToken = null;
    currentUser = null;
    localStorage.removeItem("authToken");
    localStorage.removeItem("currentUser");

    updateAuthUI();
    showToast("Logged out successfully", "success");

    if (currentView === "private" || currentView === "stats") {
      switchView("public");
    }

    notesCache.private = [];
  } catch (error) {
    console.error("Logout error:", error);
  }
}

async function loadPublicNotes() {
  try {
    elements.publicNotesGrid.innerHTML = `
            <div class="loading-spinner">
                <i class="fas fa-spinner fa-spin"></i>
                <span>Loading public notes...</span>
            </div>
        `;

    const notes = await apiRequest("/contents");
    notesCache.public = notes;
    renderPublicNotes(notes);
  } catch (error) {
    elements.publicNotesGrid.innerHTML = `
            <div class="empty-state">
                <i class="fas fa-exclamation-triangle"></i>
                <h3>Failed to load notes</h3>
                <p>Please try again later</p>
            </div>
        `;
  }
}

async function loadPrivateNotes() {
  if (!authToken) {
    elements.privateNotesGrid.innerHTML = `
            <div class="empty-state">
                <i class="fas fa-lock"></i>
                <h3>Login Required</h3>
                <p>Please login to view your private notes</p>
                <button class="btn btn-primary" onclick="showLoginModal()">
                    <i class="fas fa-sign-in-alt"></i>
                    Login
                </button>
            </div>
        `;
    return;
  }

  try {
    const notes = await apiRequest("/admin/contents");
    notesCache.private = notes;
    renderPrivateNotes(notes);
  } catch (error) {
    elements.privateNotesGrid.innerHTML = `
            <div class="empty-state">
                <i class="fas fa-exclamation-triangle"></i>
                <h3>Failed to load notes</h3>
                <p>Please try again later</p>
            </div>
        `;
  }
}

async function loadStats() {
  if (!authToken) {
    elements.statsContainer.innerHTML = `
            <div class="empty-state">
                <i class="fas fa-chart-bar"></i>
                <h3>Login Required</h3>
                <p>Please login to view your statistics</p>
                <button class="btn btn-primary" onclick="showLoginModal()">
                    <i class="fas fa-sign-in-alt"></i>
                    Login
                </button>
            </div>
        `;
    return;
  }

  try {
    const stats = await apiRequest("/admin/stats");
    renderStats(stats);
  } catch (error) {
    elements.statsContainer.innerHTML = `
            <div class="empty-state">
                <i class="fas fa-exclamation-triangle"></i>
                <h3>Failed to load statistics</h3>
                <p>Please try again later</p>
            </div>
        `;
  }
}

async function createNote(noteData) {
  try {
    const data = await apiRequest("/admin/contents", {
      method: "POST",
      body: JSON.stringify(noteData),
    });

    showToast("Note created successfully!", "success");
    elements.noteModal.classList.remove("active");
    elements.noteForm.reset();

    // Add to cache
    const newNote = {
      id: data.id,
      ...noteData,
      user: currentUser,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };

    notesCache.private.unshift(newNote);
    renderPrivateNotes(notesCache.private);
  } catch (error) {
    console.error("Create note error:", error);
  }
}

async function updateNote(id, noteData) {
  try {
    await apiRequest(`/admin/contents/${id}`, {
      method: "PUT",
      body: JSON.stringify(noteData),
    });

    showToast("Note updated successfully!", "success");
    elements.noteModal.classList.remove("active");
    elements.noteForm.reset();

    // Update cache
    const index = notesCache.private.findIndex((note) => note.id === id);
    if (index !== -1) {
      notesCache.private[index] = {
        ...notesCache.private[index],
        ...noteData,
        updated_at: new Date().toISOString(),
      };
      renderPrivateNotes(notesCache.private);
    }
  } catch (error) {
    console.error("Update note error:", error);
  }
}

async function deleteNote(id) {
  if (
    !confirm(
      "Are you sure you want to delete this note? This action cannot be undone.",
    )
  ) {
    return;
  }

  try {
    await apiRequest(`/admin/contents/${id}`, {
      method: "DELETE",
    });

    showToast("Note deleted successfully!", "success");

    // Remove from cache
    notesCache.private = notesCache.private.filter((note) => note.id !== id);
    renderPrivateNotes(notesCache.private);
  } catch (error) {
    console.error("Delete note error:", error);
  }
}

async function getNoteDetails(id) {
  try {
    const note = await apiRequest(`/admin/contents/${id}`);
    showNoteDetail(note);
  } catch (error) {
    console.error("Get note details error:", error);
  }
}

async function getPublicNoteDetails(id) {
  try {
    const note = await apiRequest(`/contents/${id}`);
    showNoteDetail(note);
  } catch (error) {
    console.error("Get public note details error:", error);
  }
}

// ===== Rendering Functions =====
function renderPublicNotes(notes) {
  if (notes.length === 0) {
    elements.publicNotesGrid.innerHTML = `
            <div class="empty-state">
                <i class="fas fa-globe"></i>
                <h3>No public notes yet</h3>
                <p>Be the first to share a note with the community!</p>
            </div>
        `;
    return;
  }

  elements.publicNotesGrid.innerHTML = notes
    .map(
      (note) => `
        <div class="note-card" onclick="showPublicNoteDetail(${note.id})">
            <div class="note-header">
                <h3 class="note-title">${escapeHtml(note.title)}</h3>
                <div class="note-meta">
                    <span class="note-author">
                        <i class="fas fa-user"></i>
                        ${escapeHtml(note.user)}
                    </span>
                    <span class="note-visibility visibility-public">
                        <i class="fas fa-globe"></i>
                        Public
                    </span>
                </div>
            </div>
            <div class="note-content">
                <p class="note-text">${escapeHtml(truncateText(note.content))}</p>
            </div>
            <div class="note-meta">
                <span class="note-date">
                    <i class="fas fa-clock"></i>
                    ${formatDate(note.created_at)}
                </span>
            </div>
        </div>
    `,
    )
    .join("");
}

function renderPrivateNotes(notes) {
  if (notes.length === 0) {
    elements.privateNotesGrid.innerHTML = `
            <div class="empty-state">
                <i class="fas fa-lock"></i>
                <h3>No private notes yet</h3>
                <p>Create your first private note to get started</p>
                <button class="btn btn-primary" onclick="showCreateModal()">
                    <i class="fas fa-plus"></i>
                    Create First Note
                </button>
            </div>
        `;
    return;
  }

  elements.privateNotesGrid.innerHTML = notes
    .map(
      (note) => `
        <div class="note-card" onclick="showNoteDetail(${note.id})">
            <div class="note-header">
                <h3 class="note-title">${escapeHtml(note.title)}</h3>
                <div class="note-meta">
                    <span class="note-author">${escapeHtml(note.user)}</span>
                    <span class="note-visibility ${note.is_public ? "visibility-public" : "visibility-private"}">
                        <i class="fas fa-${note.is_public ? "globe" : "lock"}"></i>
                        ${note.is_public ? "Public" : "Private"}
                    </span>
                </div>
            </div>
            <div class="note-content">
                <p class="note-text">${escapeHtml(truncateText(note.content))}</p>
            </div>
            <div class="note-actions">
                <button class="btn btn-secondary btn-sm" onclick="event.stopPropagation(); editNote(${note.id})">
                    <i class="fas fa-edit"></i>
                    Edit
                </button>
                <button class="btn btn-danger btn-sm" onclick="event.stopPropagation(); deleteNote(${note.id})">
                    <i class="fas fa-trash"></i>
                    Delete
                </button>
            </div>
            <div class="note-meta">
                <span class="note-date">
                    <i class="fas fa-clock"></i>
                    ${formatDate(note.created_at)}
                </span>
            </div>
        </div>
    `,
    )
    .join("");
}

function renderStats(stats) {
  elements.totalNotes.textContent = stats.total_notes || 0;
  elements.publicNotes.textContent = stats.public_notes || 0;
  elements.privateNotes.textContent = stats.private_notes || 0;
}

// ===== Modal Functions =====
function showCreateModal() {
  if (!authToken) {
    showLoginModal();
    return;
  }

  editingNoteId = null;
  elements.modalTitle.textContent = "Create New Note";
  elements.submitBtnText.textContent = "Create Note";
  elements.noteForm.reset();
  elements.noteModal.classList.add("active");
  elements.noteTitle.focus();
}

function showEditModal(noteId) {
  const note = notesCache.private.find((n) => n.id === noteId);
  if (!note) return;

  editingNoteId = noteId;
  elements.modalTitle.textContent = "Edit Note";
  elements.submitBtnText.textContent = "Update Note";

  elements.noteTitle.value = note.title;
  elements.noteContent.value = note.content;
  elements.notePublic.checked = note.is_public;

  elements.noteModal.classList.add("active");
  elements.noteTitle.focus();
}

function showLoginModal() {
  elements.loginModal.classList.add("active");
  elements.loginUsername.focus();
}

function showNoteDetail(note) {
  elements.detailTitle.textContent = note.title;
  elements.detailAuthor.textContent = `By ${escapeHtml(note.user)}`;
  elements.detailDate.textContent = formatDate(note.created_at);
  elements.detailVisibility.innerHTML = `
        <i class="fas fa-${note.is_public ? "globe" : "lock"}"></i>
        ${note.is_public ? "Public" : "Private"}
    `;
  elements.detailVisibility.className = `note-visibility ${note.is_public ? "visibility-public" : "visibility-private"}`;
  elements.detailContent.textContent = note.content;

  // Add action buttons for owned notes
  if (authToken && note.user === currentUser) {
    elements.noteActions.innerHTML = `
            <button class="btn btn-secondary" onclick="editNote(${note.id})">
                <i class="fas fa-edit"></i>
                Edit
            </button>
            <button class="btn btn-danger" onclick="deleteNote(${note.id})">
                <i class="fas fa-trash"></i>
                Delete
            </button>
        `;
  } else {
    elements.noteActions.innerHTML = "";
  }

  elements.detailModal.classList.add("active");
}

function showPublicNoteDetail(noteId) {
  const note = notesCache.public.find((n) => n.id === noteId);
  if (note) {
    showNoteDetail(note);
  } else {
    getPublicNoteDetails(noteId);
  }
}

function closeModal(modal) {
  modal.classList.remove("active");
  editingNoteId = null;
}

// ===== UI Updates =====
function updateAuthUI() {
  if (authToken && currentUser) {
    elements.loginBtn.style.display = "none";
    elements.userInfo.style.display = "flex";
    elements.username.textContent = currentUser;
    elements.createNoteBtn.style.display = "inline-flex";
  } else {
    elements.loginBtn.style.display = "flex";
    elements.userInfo.style.display = "none";
    elements.createNoteBtn.style.display = "none";
  }
}

function switchView(view) {
  currentView = view;

  // Update navigation
  elements.navBtns.forEach((btn) => {
    btn.classList.remove("active");
    if (btn.dataset.view === view) {
      btn.classList.add("active");
    }
  });

  // Hide all views
  elements.publicView.style.display = "none";
  elements.privateView.style.display = "none";
  elements.statsContainer.style.display = "none";

  // Show selected view and load data
  switch (view) {
    case "public":
      elements.publicView.style.display = "block";
      loadPublicNotes();
      break;
    case "private":
      elements.privateView.style.display = "block";
      loadPrivateNotes();
      break;
    case "stats":
      elements.statsContainer.style.display = "block";
      loadStats();
      break;
  }
}

function editNote(noteId) {
  closeModal(elements.detailModal);
  showEditModal(noteId);
}

// ===== Event Listeners =====
elements.navBtns.forEach((btn) => {
  btn.addEventListener("click", () => switchView(btn.dataset.view));
});

elements.loginBtn.addEventListener("click", showLoginModal);
elements.logoutBtn.addEventListener("click", logout);
elements.createNoteBtn.addEventListener("click", showCreateModal);

// Modal close buttons
elements.closeModal.addEventListener("click", () =>
  closeModal(elements.noteModal),
);
elements.closeLoginModal.addEventListener("click", () =>
  closeModal(elements.loginModal),
);
elements.closeDetailModal.addEventListener("click", () =>
  closeModal(elements.detailModal),
);

elements.cancelBtn.addEventListener("click", () =>
  closeModal(elements.noteModal),
);

// Close modals on background click
[elements.noteModal, elements.loginModal, elements.detailModal].forEach(
  (modal) => {
    modal.addEventListener("click", (e) => {
      if (e.target === modal) {
        closeModal(modal);
      }
    });
  },
);

// Form submissions
elements.noteForm.addEventListener("submit", async (e) => {
  e.preventDefault();

  const noteData = {
    title: elements.noteTitle.value.trim(),
    content: elements.noteContent.value.trim(),
    is_public: elements.notePublic.checked,
  };

  if (!noteData.title || !noteData.content) {
    showToast("Please fill in all required fields", "error");
    return;
  }

  if (editingNoteId) {
    await updateNote(editingNoteId, noteData);
  } else {
    await createNote(noteData);
  }
});

elements.loginForm.addEventListener("submit", async (e) => {
  e.preventDefault();

  const username = elements.loginUsername.value.trim();
  const password = elements.loginPassword.value;

  if (!username || !password) {
    showToast("Please enter both username and password", "error");
    return;
  }

  await login(username, password);
});

// Keyboard shortcuts
document.addEventListener("keydown", (e) => {
  // Escape key to close modals
  if (e.key === "Escape") {
    if (elements.noteModal.classList.contains("active")) {
      closeModal(elements.noteModal);
    } else if (elements.loginModal.classList.contains("active")) {
      closeModal(elements.loginModal);
    } else if (elements.detailModal.classList.contains("active")) {
      closeModal(elements.detailModal);
    }
  }

  // Ctrl/Cmd + N for new note
  if ((e.ctrlKey || e.metaKey) && e.key === "n") {
    e.preventDefault();
    if (authToken) {
      showCreateModal();
    } else {
      showLoginModal();
    }
  }
});

// ===== Initialize Application =====
function initApp() {
  // Load cached user data
  const cachedUser = localStorage.getItem("currentUser");
  if (cachedUser && authToken) {
    currentUser = cachedUser;
    updateAuthUI();
  }

  // Load initial view
  switchView("public");

  // Set up periodic refresh for public notes
  setInterval(() => {
    if (currentView === "public") {
      loadPublicNotes();
    }
  }, 30000); // Refresh every 30 seconds
}

// Start the application
document.addEventListener("DOMContentLoaded", initApp);
