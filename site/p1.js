/**
 * The duration in milliseconds between each frame
 */
const FRAME_DURATION_MS = 1000;
/**
 * The timout to load a frame
 */
const LOADING_TIMEOUT_MS = 5000;

/**
 * Throws a new error with the given message
 * 
 * @param {string} message The error message
 * @returns {never} This function throws and will never return
 */
function fail(message) {
    throw new Error(message);
}

/**
 * Switches the current component to the next one
 * 
 * @param {string} current The element ID of the current component
 * @param {string} next The element ID of the next component
 * @param {string} [style="block"] The style to display the next component as
 */
function switch_component(current, next, style = "block") {
    // Hide the current component
    const current_component = /** @type {HTMLElement} */
        (document.getElementById(current));
    current_component.style.display = "none";

    // Show the next component
    const next_component = /** @type {HTMLElement} */
        (document.getElementById(next));
    next_component.style.display = style;
}

/**
 * Displays the component to initialize the session
 */
function init_session() {
    // Set an on-submit handler for the form
    const init_session_form = /** @type {HTMLFormElement} */
        (document.getElementById("init-session-form"));
    init_session_form.addEventListener("submit", init_session_onsubmit);

    // Show the init session div to allow the user to log-in
    switch_component("loading", "init-session");
}

/**
 * Handles the on-submit event of the init session form
 * 
 * @param {Event} event The on-submit event
 */
function init_session_onsubmit(event) {
    // Prevent the browser from submitting the form
    event.preventDefault();

    // Load login info from form
    const address = /** @type {HTMLInputElement} */
        (document.getElementById("init-session-address"));
    const pin =  /** @type {HTMLInputElement} */
        (document.getElementById("init-session-pin"));
    const auth =  /** @type {HTMLInputElement} */
        (document.getElementById("init-session-auth"));

    // Build and encode the login info
    const login_object = { "address": address.value, "pin": pin.value, "auth": auth.value };
    const login_json = JSON.stringify(login_object);
    const login = btoa(login_json);

    // Set the hash and schedule a reload of the page
    window.location.hash = "#" + login;
    location.reload();
}

/**
 * Displays the component to play the images and starts the background-task
 * 
 * @param {string} auth The API auth token
 * @param {string} address The device address
 * @param {string} pin The device PIN
 */
function play_images(auth, address, pin) {
    // Set loading image
    const image = /** @type {HTMLImageElement} */
        (document.getElementById("play-images-image"));
    // @ts-ignore - is from `loading.js`
    image.src = LOADING_FRAME_URL;
    
    // Set the device name
    const deviceaddress = /** @type {HTMLDivElement} */
        (document.getElementById("play-images-deviceaddress"));
    deviceaddress.innerText = address;

    // Show the playback div and start the playback
    switch_component("loading", "play-images")
    setInterval(() => play_images_fetch(auth, address, pin, play_images_show), FRAME_DURATION_MS);
}

/**
 * Displays the component to play the images and starts the background-task
 * 
 * @param {string} auth The API auth token
 * @param {string} address The device address
 * @param {string} pin The device PIN
 * @param {function} onCompletion The callback function for when the task is completed
 */
function play_images_fetch(auth, address, pin, onCompletion) {
    // Build query string
    const query_string_obj = new URLSearchParams({ auth: auth, address: address, pin: pin });
    const query_string = query_string_obj.toString();
    
    // Fetch the new image
    fetch("/v1/p1?" + query_string, { method: "POST", signal: AbortSignal.timeout(5000) })
        .then(response => response.blob())
        .then(image_blob => onCompletion(image_blob))
        .catch(() => onCompletion(null));
}

/**
 * Displays the component to play the images and starts the background-task
 * 
 * @param {Blob|null} image_blob The image to show
 */
function play_images_show(image_blob) {
    // Set image
    const image = /** @type {HTMLImageElement} */
        (document.getElementById("play-images-image"));
    if (image_blob !== null && image_blob.size > 0) {
        // Set image to BLOB
        image.src = URL.createObjectURL(image_blob);
    } else {
        // Set image to loading frame
        // @ts-ignore - is from `loading.js`
        image.src = LOADING_FRAME_URL;
    }
}

/**
 * Initializes the page
 */
function init() {
    try {
        // Try to get session
        const session_hash = window.location.hash;
        if (!session_hash.startsWith("#")) {
            fail("no session available");
        }

        // Recover session object
        const session_object = atob(session_hash.substring(1));
        const session = JSON.parse(session_object);

        // If there is a session, then load it
        const address = session["address"] ?? fail("no session address");
        const pin = session["pin"] ?? fail("no session PIN");
        const auth = session["auth"] ?? '';

        // Display images
        play_images(auth, address, pin);
    } catch (e) {
        // Init session
        console.log("Failed to recover session: " + e);
        init_session();
    }
}
