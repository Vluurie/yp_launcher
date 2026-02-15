import 'package:flutter/material.dart';

/// Centralized color constants for the YoRHa Protocol Launcher
/// Using bright, warm beige/cream tones for a friendly, trustworthy appearance
class AppColors {
  AppColors._();

  // Primary background colors - warm beige tones (less bright)
  static const Color backgroundPrimary = Color(0xFFE8E4D4);    // Muted beige
  static const Color backgroundSecondary = Color(0xFFDDD8C8);  // Warmer cream
  static const Color backgroundCard = Color(0xFFF0EDE3);       // Soft cream

  // Surface colors
  static const Color surfaceLight = Color(0xFFF5F2E8);         // Soft warm white
  static const Color surfaceMedium = Color(0xFFE5E0D3);        // Light tan

  // Text colors - darker for better contrast
  static const Color textPrimary = Color(0xFF2D2D2D);          // Darker gray
  static const Color textSecondary = Color(0xFF505050);        // Medium dark gray
  static const Color textMuted = Color(0xFF686868);            // Medium gray (darkened for readability)

  // Accent colors - warm gold/amber tones (slightly darker)
  static const Color accentPrimary = Color(0xFFC49664);        // Warm gold/tan
  static const Color accentSecondary = Color(0xFFB89860);      // Golden beige
  static const Color accentHighlight = Color(0xFFD4B87B);      // Bright gold

  // Border colors
  static const Color borderLight = Color(0xFFC4B599);          // Tan border
  static const Color borderMedium = Color(0xFFA8987A);         // Medium tan border

  // Status colors
  static const Color success = Color(0xFF7CB342);              // Soft green
  static const Color error = Color(0xFFE57373);                // Soft red
  static const Color warning = Color(0xFFFFB74D);              // Soft orange

  // Special UI elements
  static const Color buttonBackground = Color(0xFFC49664);     // Gold button
  static const Color buttonText = Color(0xFFFFFDF7);           // Light text on buttons
  static const Color inputBackground = Color(0xFFF0EDE3);      // Input field bg

  // Shadow color
  static const Color shadow = Color(0x33000000);               // Slightly stronger shadow

  // Activity log specific
  static const Color logBackground = Color(0xFFF0EDE3);        // Soft cream log bg
  static const Color logHeaderBackground = Color(0xFFE5E0D3);  // Slightly darker header
  static const Color logText = Color(0xFF404040);              // Darker log text for readability
}
