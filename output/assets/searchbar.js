
const resultsDiv = document.querySelector(".results-div");
const searchInput = document.getElementById("search-input");
const tagCheckboxes = document.querySelectorAll(".tag-box input");
let last_results = [];

function isEqual(arr1, arr2) {
    // If the length is different they are not the same list
    if (arr1.length !== arr2.length) return false;
    // checks the id across the list looking for a difference
    if (!arr1.every((id, index) => id === arr2[index])) return false;
    return true;
}

function sortPosts(a, b) {
    // Sort by tag matches
    if (b.tagMatches !== a.tagMatches) return b.tagMatches - a.tagMatches;

    // If tags are equal, sort by keyword relevance
    if (b.keywordRelevance !== a.keywordRelevance) return b.keywordRelevance - a.keywordRelevance;

    // If relevance is also equal, sort by most recent date
    return new Date(b.date) - new Date(a.date);
}

// Function to filter posts into search results
function filterResults(searchTerm) {
    console.log(searchTerm);
    // get the Selected tags
    const selectedTags = Array.from(tagCheckboxes)
        .filter(checkbox => checkbox.checked)
        .map(checkbox => checkbox.value);

    // Split searchTerm into multiple terms, trimming whitespace and filtering out empty terms
    const searchTerms = searchTerm.toLowerCase().split(/\s+/).filter(term => term);

    // Filter Previews to return relevant results
    const filteredPreviews = previews
        .map(preview => {
            // Calculate tag matches
            const tagMatches = selectedTags.filter(tag => preview.tags.includes(tag)).length;

            // Check for keyword matches
            const keywordRelevance = searchTerms.reduce((count, term) => {
                const matchesTitle = preview.title.toLowerCase().includes(term) ? 1 : 0;
                const matchesDesc = preview.description.toLowerCase().includes(term) ? 1 : 0;
                return count + matchesTitle + matchesDesc;
            }, 0);

            return {
                ...preview,
                tagMatches,
                keywordRelevance,
            };
        })
        // keep only relevant results
        .filter(preview => preview.tagMatches > 0 || preview.keywordRelevance > 0)
        .sort(sortPosts)
        .map(previews => previews.id);

    // Check if results have changed
    if (!isEqual(last_results, filteredPreviews)) {
        console.log(filteredPreviews);
        renderResults(last_results, filteredPreviews);
        last_results = [...filteredPreviews];
    }
}

// Function to render posts into results-div
function renderResults(last_results, new_results) {

    // Get rid of old results
    last_results.forEach(id => {
        const postElementToRemove = resultsDiv.querySelector(`.result-item[data-id="${id}"]`);
        if (postElementToRemove) {
            postElementToRemove.remove();
        }
    });

    // Get new results
    new_results.forEach(id => {
        const postToAdd = previews.find(post => post.id === id);
        // TODO: make adding post a method taking an id
        if (postToAdd) {
            // Create a new post element and append it to the resultsDiv
            const postElement = document.createElement("div");
            postElement.className = "result-item";
            postElement.dataset.id = postToAdd.id; // Store the ID for easy reference
            postElement.innerHTML = `
                <h3>${postToAdd.title}</h3>
                <p>${postToAdd.description}</p>
                <p class="tags">Tags: ${postToAdd.tags.join(", ")}</p>
            `;
            resultsDiv.appendChild(postElement);
        }
    });
}


// search listener
searchInput.addEventListener("input", (event) => {
    const query = event.target.value.toLowerCase();
    // Call a function to filter results based on `query`
    filterResults(query);
});

let debounceTimeout;

searchInput.addEventListener("input", () => {
    clearTimeout(debounceTimeout);
    debounceTimeout = setTimeout(() => {
        filterResults(searchInput.value.toLowerCase());
    }, 300); // Delay filtering for 300ms
});

// search listener - tags
tagCheckboxes.forEach(checkbox => {
    checkbox.addEventListener("change", () => {
        filterResults(searchInput.value.toLowerCase());
    });
});