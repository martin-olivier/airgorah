# Airgorah Landing Page

A modern, responsive landing page for the Airgorah WiFi security auditing tool.

## Overview

This is a professional landing page built with vanilla HTML, CSS, and JavaScript that showcases Airgorah's features, installation methods, and documentation.

## Features

- **Modern Design**: Clean, professional UI with smooth animations and transitions
- **Responsive Layout**: Fully responsive design that works on all devices (desktop, tablet, mobile)
- **Interactive Elements**: 
  - Smooth scroll navigation
  - Copy-to-clipboard functionality for code snippets
  - Mobile hamburger menu
  - Animated feature cards and sections
  - Intersection observer for smooth scroll reveals

- **Sections**:
  - Hero section with call-to-action buttons
  - Features showcase (6 key features with icons)
  - How it works (4-step process)
  - Technology stack overview
  - Quick installation methods (Cargo, AUR, Docker)
  - System requirements
  - Documentation and resources links
  - Legal notice/warning
  - Community engagement section
  - Footer with social links

## File Structure

```
landing_page/
├── index.html      # Main HTML structure
├── styles.css      # Styling and responsive design
├── script.js       # Interactive features and animations
└── README.md       # This file
```

## Usage

### Local Development

1. Open `index.html` in your web browser
2. The page will load with all interactive features enabled
3. All styles and scripts are self-contained in a single directory

### Deployment

To deploy this landing page:

1. Copy the entire `landing_page` folder to your web server
2. Ensure all three files (index.html, styles.css, script.js) are in the same directory
3. No build process or dependencies required

## Features Breakdown

### HTML (index.html)
- Semantic HTML5 structure
- Mobile-first approach
- Proper meta tags for viewport and SEO
- Organized sections with meaningful IDs for navigation

### CSS (styles.css)
- CSS custom properties (variables) for easy theming
- CSS Grid and Flexbox for layouts
- Mobile-first responsive design with media queries
- Smooth animations and transitions
- Professional color scheme with accessibility in mind

### JavaScript (script.js)
- Mobile menu toggle with hamburger icon
- Smooth scroll behavior for anchor links
- Navbar scroll effects
- Intersection Observer API for scroll reveals
- Copy-to-clipboard for code snippets
- Accessibility features (keyboard navigation)
- Counter animations for statistics

## Customization

### Color Scheme

Edit the CSS variables in `styles.css`:

```css
:root {
    --primary-color: #0066ff;      /* Main blue */
    --secondary-color: #ff6600;    /* Orange accent */
    --dark-color: #1a1a2e;         /* Dark background */
    --light-color: #f5f5f5;        /* Light background */
    --text-color: #333;            /* Text color */
    /* ... more colors */
}
```

### Logo and Images

Replace the placeholder logo URL in `index.html`:
```html
<img src="https://via.placeholder.com/40x40?text=Air" alt="Airgorah Logo">
```

Change to your actual logo:
```html
<img src="path/to/your-logo.png" alt="Airgorah Logo">
```

### Text Content

Edit any text directly in `index.html`. All sections are clearly commented.

## Browser Support

- Chrome/Edge (latest)
- Firefox (latest)
- Safari (latest)
- Mobile browsers (iOS Safari, Chrome Mobile)

## Performance

- No external dependencies or libraries
- Fast loading with minimal CSS/JS
- Optimized animations using CSS transforms
- Intersection Observer for efficient scroll animations
- ~30KB total size (HTML + CSS + JS)

## Accessibility

- Semantic HTML structure
- ARIA labels where appropriate
- Keyboard navigation support (Escape key closes mobile menu)
- Color contrast meets WCAG standards
- Responsive text sizes

## SEO

- Semantic HTML5 structure
- Meta tags for viewport
- Meaningful heading hierarchy
- Alt text for images
- Links to GitHub and external resources

## Future Enhancements

Potential improvements:
- Add dynamic stats/download counter
- Integrate with GitHub API for latest releases
- Add dark mode toggle
- Add language localization
- Add newsletter signup form
- Add testimonials/reviews section
- Add blog section with latest updates

## License

This landing page is part of the Airgorah project, released under the MIT License.

## Contributing

To contribute improvements:
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

For more information, visit: https://github.com/martin-olivier/airgorah
